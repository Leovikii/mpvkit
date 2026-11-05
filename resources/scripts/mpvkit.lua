local utils = require('mp.utils')
local msg = require('mp.msg')
local helper = __MPVKIT_HELPER__
local config_file = mp.find_config_file('mpv.conf')
if not config_file then return end
local root = utils.split_path(config_file):gsub('[/\\]+$', '')
local function read_json(name)
    local file = io.open(utils.join_path(root, '.mpvkit/' .. name .. '.json'), 'rb')
    if not file then return nil end
    local text = file:read(65537); file:close()
    if not text or #text > 65536 then return nil end
    return utils.parse_json(text)
end
local installed, saved = read_json('install'), read_json('settings')
local generation = installed and installed.installationId or ''
local revision = saved and saved.revision or 0
local pending, saving = {}, false
local function save_next()
    if saving or #pending == 0 then return end
    local update = table.remove(pending, 1)
    saving = true
    mp.command_native_async({name='subprocess', args={helper, '--save-playback', root,
        utils.format_json(update), tostring(revision), generation}, capture_stdout=true,
        capture_stderr=true, playback_only=false}, function(success, result)
        saving = false
        local state = result and utils.parse_json(result.stdout or '')
        if success and result.status == 0 and state and state.generation and state.revision then
            generation, revision = state.generation, state.revision
        else
            msg.warn(state and state.code or 'preference helper failed')
            mp.osd_message('__SAVE_FAILED__')
        end
        save_next()
    end)
end
local choices = {
    profile = {{'power-saving','__POWER__'}, {'balanced','__BALANCED__'}, {'high-quality','__HIGH__'}},
    hdr = {{'auto','__AUTO__'}, {'on','__ON__'}, {'off','__OFF__'}},
    autoplay = {{'on','__ON__'}, {'off','__OFF__'}},
    ['when-finished'] = {{'quit','__QUIT__'}, {'keep-open','__KEEP__'}},
    speed = {{'0.5','0.5×'}, {'0.75','0.75×'}, {'1','1×'}, {'1.25','1.25×'}, {'1.5','1.5×'}, {'2','2×'}},
    ['loop-playlist'] = {{'on','__ON__'}, {'off','__OFF__'}},
    ['loop-file'] = {{'on','__ON__'}, {'off','__OFF__'}},
}
local function profile_options(item, value)
    local name = 'mpvkit-' .. (item == 'hdr' and 'hdr-' or '') .. value
    for _, profile in ipairs(mp.get_property_native('profile-list', {})) do
        if profile.name == name then return profile.options, name end
    end
end
local function equal(actual, expected, tolerance)
    if actual == nil then return false end
    if tonumber(actual) and tonumber(expected) then
        local a, b = tonumber(actual), tonumber(expected)
        return math.abs(a - b) <= (tolerance or 0) * math.max(1, math.abs(b))
    end
    return actual == expected
end
local function matches(entries)
    if not entries or #entries == 0 then return false end
    for _, entry in ipairs(entries) do
        if not equal(mp.get_property(entry.key), entry.value, 1e-6) then return false end
    end
    return true
end
local function active(item, value)
    if item == 'profile' or item == 'hdr' then return matches(profile_options(item, value)) end
    if item == 'speed' then return equal(mp.get_property('speed'), value) end
    local property = item == 'autoplay' and 'autocreate-playlist' or item == 'when-finished' and 'keep-open' or item
    local actual = mp.get_property(property)
    if actual == nil then return false end
    if item == 'when-finished' then return value == (actual == 'no' and 'quit' or 'keep-open') end
    local enabled = actual ~= 'no' and actual ~= '0'
    return value == (enabled and 'on' or 'off')
end
local function select(item, value)
    local valid = false
    for _, choice in ipairs(choices[item] or {}) do if choice[1] == value then valid = true end end
    if not valid then return end
    local entries, name
    if item == 'profile' or item == 'hdr' then entries, name = profile_options(item, value)
    else
        local property = item == 'autoplay' and 'autocreate-playlist' or item == 'when-finished' and 'keep-open' or item
        local native = value
        if item == 'autoplay' then native = value == 'on' and 'filter' or 'no'
        elseif item == 'when-finished' then native = value == 'quit' and 'no' or 'yes'
        elseif item ~= 'speed' then native = value == 'on' and 'inf' or 'no' end
        entries = {{key=property, value=native}}
    end
    if not entries or #entries == 0 then mp.osd_message('__UNAVAILABLE__'); return end
    local previous = {}
    for _, entry in ipairs(entries) do previous[entry.key] = mp.get_property(entry.key) end
    local _, err
    if name then _, err = mp.command_native({'apply-profile', name})
    else _, err = mp.command_native({'set', entries[1].key, entries[1].value}) end
    if err or not matches(entries) then
        for key, old in pairs(previous) do mp.set_property(key, old) end
        msg.warn(err or 'playback option did not apply'); mp.osd_message('__SWITCH_FAILED__'); return
    end
    if item == 'profile' or item == 'hdr' or item == 'autoplay' or item == 'when-finished' then
        pending[#pending + 1] = {item=item, value=value}
        if item == 'autoplay' then pending[#pending].value = value == 'on' end
        save_next()
    end
end
local function submenu(item, title)
    local items = {}
    for _, choice in ipairs(choices[item]) do
        items[#items + 1] = {title=choice[2], active=active(item, choice[1]), keep_open=true,
            value={'script-message-to', 'mpvkit', 'select', item, choice[1]}}
    end
    return {id=item, title=title, items=items}
end
local function action(title, binding)
    return {title=title, value={'script-binding', binding}}
end
local function menu_data()
    local items = {
        submenu('profile','__QUALITY__'), submenu('hdr','HDR'), submenu('speed','__SPEED__'),
        submenu('autoplay','__AUTOPLAY__'), submenu('when-finished','__END__'),
        {id='mode', title='__MODE__', separator=true, items={action('__SHUFFLE__','uosc/shuffle'),
            submenu('loop-playlist','__LOOP_LIST__'), submenu('loop-file','__LOOP_FILE__')}},
        action('__PLAYLIST__','uosc/items'), action('__SUBTITLES__','uosc/subtitles'),
        action('__AUDIO__','uosc/audio'), action('__CHAPTERS__','uosc/chapters'),
        action('__VIDEO__','uosc/video'), action('__EDITIONS__','uosc/editions'),
        action('__STREAM__','uosc/stream-quality'),
        action('__OPEN__','uosc/open-file'), action('__DEVICE__','uosc/audio-device'),
        action('__INFO__','stats/display-stats-toggle'), action('__FOLDER__','uosc/show-in-directory'),
        action('__CONFIG__','uosc/open-config-directory'), action('__CONSOLE__','commands/open'),
        {title='__QUIT__', value={'quit'}},
    }
    items[13].separator = true
    items[19].separator = true
    return {type='mpvkit-menu', items=items, search_submenus=true}
end
local function menu(submenu_id)
    if (not submenu_id or submenu_id == '') and mp.get_property_native('user-data/uosc/menu/type') == 'mpvkit-menu' then
        mp.commandv('script-message-to', 'uosc', 'close-menu', 'mpvkit-menu')
    else
        mp.commandv('script-message-to', 'uosc', 'open-menu', utils.format_json(menu_data()), submenu_id or '')
    end
end
local refresh_pending = false
local function refresh()
    if refresh_pending then return end
    refresh_pending = true
    mp.add_timeout(0, function()
        refresh_pending = false
        if mp.get_property_native('user-data/uosc/menu/type') == 'mpvkit-menu' then
            mp.commandv('script-message-to', 'uosc', 'update-menu', utils.format_json(menu_data()))
        end
    end)
end
mp.register_script_message('select', select)
mp.register_script_message('menu', menu)
local observed = {speed=true, ['autocreate-playlist']=true, ['keep-open']=true, ['loop-playlist']=true, ['loop-file']=true}
for _, item in ipairs({'profile','hdr'}) do
    for _, choice in ipairs(choices[item]) do
        for _, entry in ipairs(profile_options(item, choice[1]) or {}) do observed[entry.key] = true end
    end
end
for key in pairs(observed) do mp.observe_property(key, 'native', refresh) end
