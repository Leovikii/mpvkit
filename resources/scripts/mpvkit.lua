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
local context = read_json(mp.find_config_file('mpvkit-context.json'))
local catalog = read_json(mp.find_config_file('mpvkit-profiles.json'))
if not context or context.schemaVersion ~= 1 or not catalog or catalog.schemaVersion ~= 1 then
    msg.error('MpvKit context/profiles unavailable; shortcuts disabled')
    return
end
local profiles = catalog.profiles
local order = {'power-saving', 'balanced', 'high-quality'}
local labels = {['power-saving']='省电', balanced='均衡', ['high-quality']='高画质'}
local current = context.profile
local saving = false
local function manifest() return read_json(utils.join_path(context.root, '.mpvkit/install.json')) end
local generation = os.getenv('MPVKIT_GENERATION')
local function preference() return read_json(utils.join_path(context.root, '.mpvkit/playback.json')) end
local function publish()
    mp.set_property_native('user-data/mpvkit', {profile=current, saving=saving})
end
local function apply(id)
    if not profiles[id] then mp.osd_message('不支持的 MpvKit 档位'); return false end
    local old, keys = {}, {}
    for key in pairs(profiles[id]) do keys[#keys + 1] = key; old[key] = mp.get_property(key) end
    table.sort(keys)
    for _, key in ipairs(keys) do
        local ok, err = mp.set_property(key, profiles[id][key])
        if not ok then
            for _, restore in ipairs(keys) do if old[restore] then mp.set_property(restore, old[restore]) end end
            msg.error('profile switch failed: ' .. tostring(err))
            mp.osd_message('档位切换失败，已恢复原设置'); return false
        end
    end
    current = id; publish()
    mp.osd_message('MpvKit：' .. labels[id] .. '（仅本次；Alt+s 保存全局）')
    return true
end
local function save()
    if saving or not profiles[current] then mp.osd_message('请先选择一个档位'); return end
    local installed = manifest()
    local previous = preference()
    if not installed or installed.owner ~= 'MpvKit' or installed.schemaVersion ~= 1 then mp.osd_message('安装记录不可用，未保存'); return end
    if previous and (previous.schemaVersion ~= 1 or previous.owner ~= 'MpvKit' or type(previous.revision) ~= 'number') then mp.osd_message('偏好记录无效，未覆盖'); return end
    if not generation or generation == '' or installed.installationId ~= generation then
        msg.warn('preference_stale')
        mp.osd_message('安装或设置已变化，请重新启动播放器后保存。'); return
    end
    local revision = previous and previous.revision or 0
    saving = true; publish()
    mp.command_native_async({name='subprocess', args={context.helper, '--save-playback', context.root, current, tostring(revision), generation}, capture_stdout=true, capture_stderr=true, playback_only=false}, function(success, result)
        saving = false; publish()
        if success and result.status == 0 then
            mp.osd_message('已保存为全局默认；新启动播放器生效')
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
    items[#items + 1] = {title='保存当前档位为全局默认', hint='Alt+s', value={'script-message-to', 'mpvkit', 'save'}}
    mp.commandv('script-message-to', 'uosc', 'open-menu', utils.format_json({type='mpvkit-profiles', title='MpvKit 画质档位', items=items}))
end
mp.register_script_message('profile', apply)
mp.register_script_message('save', save)
mp.register_script_message('menu', menu)
mp.add_key_binding('Alt+1', 'power-saving', function() apply('power-saving') end)
mp.add_key_binding('Alt+2', 'balanced', function() apply('balanced') end)
mp.add_key_binding('Alt+3', 'high-quality', function() apply('high-quality') end)
mp.add_key_binding('Alt+s', 'save', save)
mp.add_key_binding('F6', 'menu', menu)
mp.add_timeout(0, function()
    local installed, saved = manifest(), preference()
    if saved and installed and saved.schemaVersion == 1 and saved.owner == 'MpvKit' and saved.generation == generation and installed.installationId == generation and profiles[saved.profile] then current = saved.profile end
    -- Initial values are set by the launcher before optional user.conf. Never overwrite it here.
    publish()
end)

-- A temporary choice belongs to one file. Later files re-read the saved default.
local first_file = true
local initial = {}
for key in pairs(profiles[current] or {}) do initial[key] = mp.get_property(key) end
mp.register_event('file-loaded', function()
    if first_file then first_file = false; return end
    local installed, saved = manifest(), preference()
    if saved and installed and saved.schemaVersion == 1 and saved.owner == 'MpvKit'
        and saved.generation == installed.installationId and profiles[saved.profile] then
        apply(saved.profile)
    else
        for key, value in pairs(initial) do mp.set_property(key, value) end
        current = context.profile; publish()
    end
end)
