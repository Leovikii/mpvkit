local msg = require('mp.msg')
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
