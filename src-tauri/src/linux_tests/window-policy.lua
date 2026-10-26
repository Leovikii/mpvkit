-- Headless behavior checks against the actual startup policy; no user config is loaded.
local source = assert(io.open('resources/scripts/mpvkit.lua', 'rb'))
local text = source:read('*a'); source:close()
local start = assert(text:find('-- Wayland initially', 1, true))
local policy = text:sub(start)
local function scenario(overrides, platform)
    local props = {platform=platform or 'linux', geometry='', ['window-scale']=1, autofit='60%x60%',
        ['osd-dimensions']={w=1080,h=607}, ['display-names']={}, ['vo-configured']=false,
        ['current-gpu-context']='waylandvk'}
    for key, value in pairs(overrides or {}) do props[key]=value end
    local observers, writes = {}, {}
    local api = {
        get_property=function(key, default) return props[key] == nil and default or props[key] end,
        get_property_number=function(key, default) return props[key] == nil and default or props[key] end,
        get_property_bool=function(key) return props[key] == true end,
        get_property_native=function(key) return props[key] end,
        observe_property=function(_, _, fn) observers[fn]=true end,
        unobserve_property=function(fn) observers[fn]=nil end,
        set_property=function(key,value) writes[#writes+1]={key,value} end,
    }
    local env = setmetatable({mp=api, os={getenv=function() return 'wayland-0' end}, msg={info=function() end}}, {__index=_G})
    local fn
    if setfenv then fn=assert(loadstring(policy));setfenv(fn,env)
    else fn=assert(load(policy,'startup-policy','t',env)) end
    fn()
    local function notify(changes)
        for key,value in pairs(changes or {}) do props[key]=value end
        local pending={};for callback in pairs(observers) do pending[#pending+1]=callback end
        for _,callback in ipairs(pending) do callback() end
    end
    return props,writes,notify,function() return next(observers) == nil end
end
local ready={['display-names']={'DP-1'}, ['vo-configured']=true, ['video-out-params']={w=1280,h=720}}
local function complete(notify) notify(ready) end
local function check()
    local props,writes,notify,stopped=scenario()
    notify();assert(#writes==0 and not stopped())
    notify({['osd-dimensions']={w=1080,h=608}});complete(notify)
    assert(#writes==1 and writes[1][1]=='autofit' and writes[1][2]=='60%x60%' and stopped())
    notify({['osd-dimensions']={w=1600,h=900}, ['display-names']={'DP-2'}});assert(#writes==1)
    for _,override in ipairs({{geometry='900x506'},{['window-scale']=0.5},{fullscreen=true},
        {['window-maximized']=true},{['window-minimized']=true},{['auto-window-resize']=true}}) do
        local _,w,n,done=scenario(override);n();complete(n);assert(#w==0 and done())
    end
    local _,w,n,done=scenario();n();n({['osd-dimensions']={w=900,h=506}});complete(n)
    assert(#w==0 and done(), 'user resize before output must cancel')
    local _,w,n,done=scenario();n();n({['display-names']={'DP-1','DP-2'}, ['vo-configured']=true, ['video-out-params']={w=1280,h=720}})
    complete(n);assert(#w==0 and done(), 'ambiguous output must cancel permanently')
    local _,w,n,done=scenario(nil,'windows');complete(n);assert(#w==0 and done())
    local _,w,n,done=scenario({['current-gpu-context']='x11'});complete(n);assert(#w==0 and done())
    local _,w,n=scenario();n({['display-names']={'DP-1'}, ['vo-configured']=true});assert(#w==0)
    n({['video-out-params']={w=1280,h=720}});assert(#w==1, 'audio first waits for video')
end
local success,err=pcall(check)
if success then mp.msg.info('PASS: startup once, effective autofit, rounding, explicit size, user resize, ambiguous outputs, platform/context and audio-first')
else mp.msg.error(tostring(err)) end
mp.commandv('quit', success and '0' or '1')
