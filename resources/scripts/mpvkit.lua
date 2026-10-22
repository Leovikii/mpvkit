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
local options = {profile='balanced', screenshot_subtitles=false}
require('mp.options').read_options(options, 'mpvkit')
local config_file = mp.find_config_file('mpv.conf')
if not config_file then return end
local config_dir = utils.split_path(config_file)
local root = utils.split_path(config_dir:gsub('[/\\]+$', ''))
local helper = utils.join_path(root, '.mpvkit/launcher/mpvkit.exe')
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
mp.register_script_message('screenshot', function()
    mp.command_native_async({name='screenshot', flags=options.screenshot_subtitles and 'subtitles' or 'video'}, function(success, result, err)
        if not success then mp.osd_message('截图失败 / Screenshot failed'); msg.error(tostring(err)) end
    end)
end)
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
