-- Headless checks run the actual policy with a deterministic clock and no user config.
local source = assert(io.open('resources/scripts/mpvkit.lua', 'rb'))
local text = source:read('*a'); source:close()
local start = assert(text:find('-- Wayland initially', 1, true))
local policy = text:sub(start)
local function scenario(overrides, platform)
    local props = {platform=platform or 'linux', geometry='', ['window-scale']=1, autofit='60%x60%',
        ['osd-dimensions']={w=1080,h=607}, ['display-names']={}, ['vo-configured']=false,
        ['current-gpu-context']='waylandvk'}
    for key, value in pairs(overrides or {}) do props[key]=value end
    local observers, writes, events, timers, now = {}, {}, {}, {}, 0
    local api = {
        get_property=function(key, default) return props[key] == nil and default or props[key] end,
        get_property_number=function(key, default) return props[key] == nil and default or props[key] end,
        get_property_bool=function(key) return props[key] == true end,
        get_property_native=function(key) return props[key] end,
        observe_property=function(_, _, fn) observers[fn]=true end,
        unobserve_property=function(fn) observers[fn]=nil end,
        register_event=function(name, fn) events[name]=fn end,
        unregister_event=function(fn) for name, callback in pairs(events) do if callback==fn then events[name]=nil end end end,
        set_property=function(key,value) writes[#writes+1]={key,value} end,
        add_timeout=function(delay, fn)
            local timer={at=now+delay, callback=fn, kill=function(self) self.killed=true end}
            timers[#timers+1]=timer;return timer
        end,
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
    local function advance(seconds)
        now=now+seconds
        local pending=timers;timers={}
        for _,timer in ipairs(pending) do
            if not timer.killed then
                if timer.at<=now then timer.callback() else timers[#timers+1]=timer end
            end
        end
    end
    return props,writes,notify,function() return next(observers)==nil and next(events)==nil end,advance,
        function(name) if events[name] then events[name]() end end
end
local ready={['display-names']={'DP-1'}, ['vo-configured']=true, ['video-out-params']={w=1280,h=720}}
local function complete(notify) notify(ready) end
local function check()
    local props,writes,notify,stopped,advance=scenario()
    notify();assert(#writes==0 and not stopped())
    complete(notify);assert(#writes==0 and not stopped(), 'surface-enter must not resize immediately')
    advance(.4);notify({['osd-dimensions']={w=1080,h=608}})
    advance(.4);assert(#writes==0, 'rounding/configure restarts the quiet period')
    props.autofit='75%x75%' -- The callback must re-read effective options, even before observer delivery.
    advance(.11)
    assert(#writes==1 and writes[1][1]=='autofit' and writes[1][2]=='75%x75%' and stopped())
    notify({['osd-dimensions']={w=1600,h=900}, ['display-names']={'DP-2'}});advance(5);assert(#writes==1)
    for _,override in ipairs({{geometry='900x506'},{['window-scale']=0.5},{fullscreen=true},
        {['window-maximized']=true},{['window-minimized']=true},{['auto-window-resize']=true},
        {['current-gpu-context']='x11'},{autofit=''}}) do
        local _,w,n,done,t=scenario(override);n();complete(n);t(1);assert(#w==0 and done())
        local _,w,n,done,t=scenario();complete(n);t(.2);n(override);t(1)
        assert(#w==0 and done(), 'late override must cancel the pending correction')
    end
    local _,w,n,done,t=scenario();complete(n);t(.2);n({['osd-dimensions']={w=900,h=506}});t(1)
    assert(#w==0 and done(), 'user resize while settling must cancel')
    local props,w,n,done,t=scenario();complete(n);props.geometry='900x506';t(1)
    assert(#w==0 and done(), 'revalidate even if a property notification has not arrived')
    local _,w,n,done,t=scenario();n();n({['display-names']={'DP-1','DP-2'}, ['vo-configured']=true, ['video-out-params']={w=1280,h=720}})
    complete(n);t(1);assert(#w==0 and done(), 'ambiguous output must cancel permanently')
    local _,w,n,done,t=scenario();complete(n);n({['display-names']={'DP-2'}});t(1)
    assert(#w==0 and done(), 'moving to another output while settling must cancel')
    local _,w,n,done,t,event=scenario();complete(n);event('start-file');t(1)
    assert(#w==0 and done(), 'changing files must cancel the pending correction')
    local _,w,n,done,t=scenario(nil,'windows');complete(n);t(1);assert(#w==0 and done())
    local props,w,n,done,t,event=scenario({['osd-dimensions']={w=0,h=0}})
    event('start-file');n({['display-names']={'DP-1'}, ['vo-configured']=true});t(1);assert(#w==0)
    event('start-file');n({['video-out-params']={w=1280,h=720},['osd-dimensions']={w=1080,h=608}})
    t(.51);assert(#w==1 and done(), 'audio first waits for the first video')
end
local success,err=pcall(check)
if success then mp.msg.info('PASS: quiet startup, live revalidation, cancellation, once-only correction and audio-first')
else mp.msg.error(tostring(err)) end
mp.commandv('quit', success and '0' or '1')
