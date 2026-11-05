-- Deterministic checks of the deployed selection/save flow; no user configuration is read.
local utils = require('mp.utils')
local file = assert(io.open('resources/scripts/mpvkit.lua','rb'))
local source = file:read('*a'):gsub('__MPVKIT_HELPER__', "'fixture-helper'"); file:close()
local props = {scale='lanczos', ['target-trc']='auto', ['autocreate-playlist']='filter', ['keep-open']='no', speed='1'}
local profiles = {}
for name,values in pairs({['power-saving']={scale='bilinear'}, balanced={scale='lanczos'},
    ['high-quality']={scale='ewa_lanczossharp'}, ['hdr-auto']={['target-trc']='auto'},
    ['hdr-on']={['target-trc']='pq'}, ['hdr-off']={['target-trc']='srgb'}}) do
    local entries={};for key,value in pairs(values) do entries[#entries+1]={key=key,value=value} end
    profiles[#profiles+1]={name='mpvkit-'..name,options=entries}
end
local messages,jobs,osd,observers = {},{},{},{}
local fail = false
local api = {
    find_config_file=function() return '/fixture/mpv.conf' end,
    get_property=function(key) return props[key] end,
    get_property_native=function(key) if key=='profile-list' then return profiles end return props[key] end,
    set_property=function(key,value) props[key]=value end,
    observe_property=function(key,_,fn) observers[key]=fn end,
    register_script_message=function(name,fn) messages[name]=fn end,
    osd_message=function(text) osd[#osd+1]=text end,
    add_timeout=function(_,fn) fn() end,
    commandv=function() end,
    command_native=function(command)
        if command[1]=='apply-profile' then
            for _,p in ipairs(profiles) do if p.name==command[2] then
                for _,entry in ipairs(p.options) do props[entry.key]=entry.value end
            end end
        else props[command[2]]=command[3] end
        if fail then return nil,'fixture failure' end
        return true
    end,
    command_native_async=function(command,callback) jobs[#jobs+1]={command=command,callback=callback} end,
}
local env=setmetatable({mp=api,require=function(name) return name=='mp.utils' and utils or {warn=function() end} end,
    io={open=function(path) return {read=function() return path:find('install',1,true) and '{"installationId":"initial"}' or '{"revision":1}' end, close=function() end} end}}, {__index=_G})
local fn
if setfenv then fn=assert(loadstring(source));setfenv(fn,env) else fn=assert(load(source,'playback-policy','t',env)) end
local function check()
    fn()
    messages.select('profile','high-quality');messages.select('hdr','on');messages.select('autoplay','off');messages.select('when-finished','keep-open')
    assert(#jobs==1,'requests must be serialized')
    for i,item in ipairs({'profile','hdr','autoplay','when-finished'}) do
        assert(#jobs==i)
        local job=jobs[i];local update=utils.parse_json(job.command.args[4])
        assert(update.item==item);if item=='autoplay' then assert(update.value==false) end
        assert(job.command.args[5]==tostring(i));assert(job.command.args[6]==(i==1 and 'initial' or 'generation-'..(i-1)))
        job.callback(true,{status=0,stdout=utils.format_json({revision=i+1,generation='generation-'..i})})
    end
    assert(props.scale=='ewa_lanczossharp' and props['target-trc']=='pq')
    assert(props['autocreate-playlist']=='no' and props['keep-open']=='yes')
    -- A failed native application rolls back and never queues a save.
    fail=true;messages.select('profile','power-saving');fail=false
    assert(props.scale=='ewa_lanczossharp' and #jobs==4)
    messages.select('volume','50');messages.select('hdr','injected');assert(#jobs==4)
    -- A save failure does not mark the runtime value as saved or prevent a retry.
    messages.select('hdr','off');jobs[5].callback(false,{status=1,stdout='{"code":"file_conflict"}'})
    assert(props['target-trc']=='srgb' and #osd>=2)
    messages.select('hdr','off');assert(#jobs==6 and jobs[6].command.args[5]=='5')
    jobs[6].callback(true,{status=0,stdout='{"revision":6,"generation":"generation-6"}'})
    props.speed='1.1';observers.speed();assert(#jobs==6,'observation must not save')
    assert(messages.save==nil and messages.profile==nil,'old quality-only messages are removed')
end
local success,err=pcall(check)
if success then mp.msg.info('PASS: FIFO saves, four controlled items, native rollback, failed-save retry and read-only observers')
else mp.msg.error(tostring(err)) end
mp.commandv('quit',success and '0' or '1')
