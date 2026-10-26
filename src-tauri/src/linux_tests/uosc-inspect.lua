-- Private test instrumentation appended only to the isolated native copy.
local files_loaded = 0
mp.register_event('file-loaded', function()
    files_loaded = files_loaded + 1
    mp.set_property_native('user-data/mpvkit-test-loaded', files_loaded)
end)
-- IPC mouse doesn't mark Wayland hover=true outside the physical pointer's window.
-- Drive the same native cursor entry point without moving the user's real pointer.
mp.register_script_message('mpvkit-test-pointer', function(x, y)
    cursor.last_hover = false
    cursor:move(tonumber(x), tonumber(y))
end)
mp.register_script_message('mpvkit-test-inspect', function()
    local controls = {}
    for i, control in ipairs(Elements.controls.controls) do
        local e = control.element
        if e then controls[tostring(i)] = {enabled=e.enabled, hidden=control.hide, ax=e.ax, ay=e.ay,
            bx=e.bx, by=e.by, icon=e.icon, name=e.name, font=e.font_size, kind=control.kind,
            visibility=e:get_visibility(), text=e.name == 'mpvkit-speed' and string.format('%g×', mp.get_property_number('speed', 1)) or nil} end
    end
    local menu = Elements.menu
    local items = {}
    if menu then for _, item in ipairs(menu.current.items) do items[#items+1] = {title=item.title, hint=item.hint} end end
    mp.set_property_native('user-data/mpvkit-test', {width=display.width, height=display.height, scale=state.scale, curtain=Elements.curtain.opacity,
        controls=controls, menu=menu and {type=menu.type, items=items} or false})
end)
