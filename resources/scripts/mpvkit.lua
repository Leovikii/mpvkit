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
local labels = {['power-saving']='省电', balanced='均衡', ['high-quality']='高画质'}
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
    if not entries then mp.osd_message('画质配置不可用'); return false end
    local old = {}
    for _, entry in ipairs(entries) do old[entry.key] = mp.get_property(entry.key) end
    local _, err = mp.command_native({'apply-profile', 'mpvkit-' .. id})
    if err then
        for key, value in pairs(old) do mp.set_property(key, value) end
        msg.error(tostring(err)); mp.osd_message('档位切换失败，已恢复原设置'); return false
    end
    current=id;publish();mp.osd_message('MpvKit：' .. labels[id] .. '（仅本次）');return true
end
local function save()
    if saving or not labels[current] then mp.osd_message('请先选择一个档位'); return end
    local installed = manifest()
    local previous = preference()
    if not installed or installed.owner ~= 'MpvKit' or installed.schemaVersion ~= 1 then mp.osd_message('安装记录不可用，未保存'); return end
    if previous and (previous.schemaVersion ~= 1 or previous.owner ~= 'MpvKit' or type(previous.revision) ~= 'number') then mp.osd_message('偏好记录无效，未覆盖'); return end
    if not generation or generation == '' or installed.installationId ~= generation then
        msg.warn('preference_stale')
        mp.osd_message('安装或设置已变化，请重新启动播放器后保存。'); return
    end
    if previous and previous.revision ~= revision then mp.osd_message('设置已变化，请重新启动播放器。'); return end
    saving = true; publish()
    mp.command_native_async({name='subprocess', args={helper, '--save-playback', root, current, tostring(revision), generation}, capture_stdout=true, capture_stderr=true, playback_only=false}, function(success, result)
        saving = false; publish()
        if success and result.status == 0 then
            local result_state = utils.parse_json(result.stdout or '')
            if result_state then generation=result_state.generation; revision=result_state.revision end
            mp.osd_message('已保存为默认；下次启动生效')
        else
            local failure
            if result and result.stdout then failure = utils.parse_json(result.stdout) end
            mp.osd_message(failure and failure.message or '全局保存失败；本次选择仍然有效')
            msg.warn(failure and failure.code or 'preference helper failed')
        end
    end)
end
local function menu()
    local items = {}
    for _, id in ipairs(order) do items[#items + 1] = {title=labels[id], hint=current == id and '当前' or '仅本次', value={'script-message-to', 'mpvkit', 'profile', id}} end
    items[#items + 1] = {title='保存当前档位为全局默认', hint='', value={'script-message-to', 'mpvkit', 'save'}}
    mp.commandv('script-message-to', 'uosc', 'open-menu', utils.format_json({type='mpvkit-profiles', title='MpvKit 画质档位', items=items}))
end
mp.register_script_message('profile', apply)
mp.register_script_message('save', save)
mp.register_script_message('menu', menu)
mp.add_timeout(0, publish)

-- Playback modes use the bridge's real uosc state and MPV's loop properties.
-- No mode is persisted; external loop combinations remain visible as custom.
local modes = {
    {id='sequential', label='顺序播放 / Sequential', icon='arrow_forward', loop_file='no', loop_playlist='no', shuffle=false},
    {id='playlist', label='列表循环 / Loop playlist', icon='repeat', loop_file='no', loop_playlist='inf', shuffle=false},
    {id='file', label='单集循环 / Loop file', icon='repeat_one', loop_file='inf', loop_playlist='no', shuffle=false},
    {id='shuffle', label='随机播放 / Shuffle', icon='shuffle', loop_file='no', loop_playlist='no', shuffle=true},
}
local function playback_mode()
    local actual = mp.get_property_native('user-data/mpvkit-uosc')
    if type(actual) ~= 'table' or type(actual.shuffle) ~= 'boolean' then return nil end
    local file = mp.get_property('loop-file', 'no')
    local playlist = mp.get_property('loop-playlist', 'no')
    for _, mode in ipairs(modes) do
        if actual.shuffle == mode.shuffle and file == mode.loop_file and playlist == mode.loop_playlist then return mode end
    end
    return {id='custom', label='自定义 / Custom', icon='tune'}
end
local function playback_menu_data()
    local current_mode = playback_mode()
    local items = {}
    for _, mode in ipairs(modes) do
        items[#items + 1] = {title=mode.label, hint=current_mode and current_mode.id == mode.id and '当前 / Current' or '',
            value={'script-message-to', 'mpvkit', 'playback-mode', mode.id}}
    end
    return {type='mpvkit-playback', title='播放模式 / Playback mode',
        footnote=current_mode and current_mode.id == 'custom' and current_mode.label or nil, items=items}
end
local function update_playback_mode()
    local mode = playback_mode()
    mp.set_property_native('user-data/mpvkit-playback-mode', mode and mode.id or 'unavailable')
    mp.commandv('script-message-to', 'uosc', 'set-button', 'mpvkit-mode', utils.format_json({
        icon=mode and mode.icon or 'help_outline', active=mode and mode.id ~= 'sequential' or false,
        tooltip='播放模式 / Playback mode' .. (mode and ' · ' .. mode.label or ''),
        command=mode and {'script-message-to', 'mpvkit', 'playback-menu'} or nil,
    }))
    mp.commandv('script-message-to', 'uosc', 'update-menu', utils.format_json(playback_menu_data()))
end
mp.register_script_message('playback-menu', function()
    if not playback_mode() then mp.osd_message('uosc 尚未就绪 / uosc is not ready'); return end
    mp.commandv('script-message-to', 'uosc', 'open-menu', utils.format_json(playback_menu_data()))
end)
mp.register_script_message('playback-mode', function(id)
    if not playback_mode() then return end
    for _, mode in ipairs(modes) do
        if mode.id == id then
            mp.set_property('loop-file', mode.loop_file)
            mp.set_property('loop-playlist', mode.loop_playlist)
            mp.commandv('script-message-to', 'uosc', 'mpvkit-set-shuffle', mode.shuffle and 'yes' or 'no')
            return
        end
    end
end)
local function speed_menu_data()
    local items = {}
    local current_speed = mp.get_property_number('speed', 1)
    for _, speed in ipairs({0.5, 0.75, 1, 1.25, 1.5, 2}) do
        items[#items + 1] = {title=tostring(speed) .. '×', hint=current_speed == speed and '当前 / Current' or '',
            value={'set', 'speed', tostring(speed)}}
    end
    return {type='mpvkit-speed', title='倍速 / Playback speed', items=items}
end
local function update_speed()
    mp.commandv('script-message-to', 'uosc', 'set-button', 'mpvkit-speed', utils.format_json({
        icon='speed', tooltip='倍速 / Playback speed · ' .. string.format('%g×', mp.get_property_number('speed', 1)),
        command={'script-message-to', 'mpvkit', 'speed-menu'},
    }))
    mp.commandv('script-message-to', 'uosc', 'update-menu', utils.format_json(speed_menu_data()))
end
mp.register_script_message('speed-menu', function()
    mp.commandv('script-message-to', 'uosc', 'open-menu', utils.format_json(speed_menu_data()))
end)
mp.observe_property('speed', 'number', update_speed)
mp.observe_property('user-data/mpvkit-uosc', 'native', function() update_playback_mode(); update_speed() end)
mp.observe_property('loop-file', 'string', update_playback_mode)
mp.observe_property('loop-playlist', 'string', update_playback_mode)

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
