local utils = require('mp.utils')
local msg = require('mp.msg')
local function read_json(path)
    if not path then return nil end
    local file = io.open(path, 'rb')
    if not file then return nil end
    local content = file:read(65537); file:close()
    if not content or #content > 65536 then return nil end
    return utils.parse_json(content)
end
local options = {profile='balanced'}
require('mp.options').read_options(options, 'mpvkit')
local config_file = mp.find_config_file('mpv.conf')
if not config_file then return end
local config_dir = utils.split_path(config_file)
local root = config_dir:gsub('[/\\]+$', '')
local helper = __MPVKIT_HELPER__
local order = {'power-saving', 'balanced', 'high-quality'}
local labels = {['power-saving']='__POWER__', balanced='__BALANCED__', ['high-quality']='__HIGH__'}
local current = options.profile
local saving = false
local function manifest() return read_json(utils.join_path(root, '.mpvkit/install.json')) end
local function preference() return read_json(utils.join_path(root, '.mpvkit/settings.json')) end
local installed = manifest()
local generation = installed and installed.installationId
local saved_at_start = preference()
local revision = saved_at_start and saved_at_start.revision or 0
local function profile_options(id)
    for _, p in ipairs(mp.get_property_native('profile-list', {})) do
        if p.name == 'mpvkit-' .. id then return p.options end
    end
end
local function publish()
    mp.set_property_native('user-data/mpvkit', {profile=current, saving=saving})
end
local function apply(id)
    local entries = labels[id] and profile_options(id)
    if not entries then mp.osd_message('__UNAVAILABLE__'); return false end
    local old = {}
    for _, entry in ipairs(entries) do old[entry.key] = mp.get_property(entry.key) end
    local _, err = mp.command_native({'apply-profile', 'mpvkit-' .. id})
    if err then
        for key, value in pairs(old) do mp.set_property(key, value) end
        msg.error(tostring(err)); mp.osd_message('__SWITCH_FAILED__'); return false
    end
    current=id;publish();mp.osd_message(labels[id] .. ' (__TEMPORARY__)');return true
end
local function save()
    if saving or not labels[current] then mp.osd_message('__CHOOSE__'); return end
    local installed = manifest()
    local previous = preference()
    if not installed or installed.owner ~= 'MpvKit' or installed.schemaVersion ~= 1 then mp.osd_message('__RECORD_INVALID__'); return end
    if previous and (previous.schemaVersion ~= 1 or previous.owner ~= 'MpvKit' or type(previous.revision) ~= 'number') then mp.osd_message('__PREFERENCE_INVALID__'); return end
    if not generation or generation == '' or installed.installationId ~= generation then
        msg.warn('preference_stale')
        mp.osd_message('__STALE__'); return
    end
    if previous and previous.revision ~= revision then mp.osd_message('__STALE__'); return end
    saving = true; publish()
    mp.command_native_async({name='subprocess', args={helper, '--save-playback', root, current, tostring(revision), generation}, capture_stdout=true, capture_stderr=true, playback_only=false}, function(success, result)
        saving = false; publish()
        if success and result.status == 0 then
            local result_state = utils.parse_json(result.stdout or '')
            if result_state then generation=result_state.generation; revision=result_state.revision end
            mp.osd_message('__SAVED__')
        else
            local failure
            if result and result.stdout then failure = utils.parse_json(result.stdout) end
            mp.osd_message(failure and failure.code == 'preference_stale' and '__STALE__' or '__SAVE_FAILED__')
            msg.warn(failure and failure.code or 'preference helper failed')
        end
    end)
end
local function menu()
    local items = {}
    for _, id in ipairs(order) do items[#items + 1] = {title=labels[id], hint=current == id and '__CURRENT__' or '__TEMPORARY__', value={'script-message-to', 'mpvkit', 'profile', id}} end
    items[#items + 1] = {title='__SAVE__', hint='', value={'script-message-to', 'mpvkit', 'save'}}
    mp.commandv('script-message-to', 'uosc', 'open-menu', utils.format_json({type='mpvkit-profiles', title='__QUALITY__', items=items}))
end
mp.register_script_message('profile', apply)
mp.register_script_message('save', save)
mp.register_script_message('menu', menu)
mp.add_timeout(0, publish)

-- Temporary profile changes belong to this media file; preserve the actual initial configuration.
local initial = {}
for _, id in ipairs(order) do
    for _, entry in ipairs(profile_options(id) or {}) do initial[entry.key] = mp.get_property(entry.key) end
end
local first_file = true
mp.register_event('file-loaded', function()
    if first_file then first_file=false; return end
    for key,value in pairs(initial) do mp.set_property(key,value) end
    current=options.profile;publish()
end)

-- Wayland initially sizes against its first output before surface-enter identifies
-- the actual output. Reapply the effective autofit once; never resize on episode changes.
if mp.get_property('platform') == 'linux' and os.getenv('WAYLAND_DISPLAY') then
    local first_size, first_output, finished, settle_timer
    local check_window, next_file
    local function stop()
        finished = true
        if settle_timer then settle_timer:kill(); settle_timer = nil end
        mp.unobserve_property(check_window)
        mp.unregister_event(next_file)
    end
    local function ready()
        if finished then return end
        if mp.get_property('geometry', '') ~= '' or mp.get_property_number('window-scale', 1) ~= 1
            or mp.get_property_bool('fullscreen') or mp.get_property_bool('window-maximized')
            or mp.get_property_bool('window-minimized') or mp.get_property_bool('auto-window-resize') then
            stop(); return
        end
        local size = mp.get_property_native('osd-dimensions')
        if not size or size.w <= 0 or size.h <= 0 then return end
        if first_size and (math.abs(size.w - first_size.w) > 2 or math.abs(size.h - first_size.h) > 2) then
            stop(); return -- Respect size changes while waiting for the first real output.
        end
        first_size = first_size or size
        local names = mp.get_property_native('display-names')
        local video = mp.get_property_native('video-out-params')
        if not mp.get_property_bool('vo-configured') or not names or #names == 0 or not video then return end
        if #names ~= 1 then
            stop()
            msg.info('Skipped startup autofit: window spans multiple Wayland outputs')
            return
        end
        if first_output and first_output ~= names[1] then stop(); return end
        first_output = names[1]
        local context = mp.get_property('current-gpu-context', '')
        local fit = mp.get_property('autofit', '')
        if (context == 'waylandvk' or context == 'wayland') and fit ~= '' then
            return fit
        end
        stop()
    end
    check_window = function()
        if settle_timer then settle_timer:kill(); settle_timer = nil end
        if not ready() then return end
        -- shortcut: Lua has no configure-complete signal; revalidate after 0.5s quiet.
        -- Prefer a native completion signal when MPV provides one.
        settle_timer = mp.add_timeout(0.5, function()
            settle_timer = nil
            local fit = ready()
            if not fit then return end
            stop()
            mp.set_property('autofit', fit)
            msg.info('Reapplied startup autofit on the actual Wayland output')
        end)
    end
    next_file = function()
        if first_size then stop() end
    end
    mp.register_event('start-file', next_file)
    for _, name in ipairs({'display-names', 'osd-dimensions', 'video-out-params', 'vo-configured',
        'geometry', 'window-scale', 'fullscreen', 'window-maximized', 'window-minimized',
        'auto-window-resize', 'autofit', 'current-gpu-context'}) do
        mp.observe_property(name, 'native', check_window)
    end
end
