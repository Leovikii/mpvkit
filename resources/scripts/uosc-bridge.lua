
-- MpvKit integration, 2026-10-26, GPL-3.0-or-later.
-- Appended to the verified uosc 5.13.0 main.lua; upstream bytes remain an unchanged prefix.
-- Use uosc's actual shuffle state and dimensions, rather than duplicating its navigation.
do
    local function publish_layout()
        local unit = math.max(1, options.controls_size * state.scale)
        local width = math.max(0, display.width - 2 * options.controls_margin * state.scale) / unit
        mp.set_property_native('user-data/mpvkit-uosc', {
            shuffle=state.shuffle, info=width >= 20, tracks=width >= 14, queue=width >= 10,
        })
    end
    local on_shuffle = state.on_shuffle
    state.on_shuffle = function(value)
        if on_shuffle then on_shuffle(value) end
        publish_layout()
    end
    mp.register_script_message('mpvkit-set-shuffle', function(value)
        if value ~= 'yes' and value ~= 'no' then return end
        set_state('shuffle', value == 'yes')
        request_render()
    end)
    local Element = require('elements/Element')
    local Layout = class(Element)
    function Layout:init() Element.init(self, 'mpvkit_layout', {enabled=false}) end
    function Layout:on_display() publish_layout() end
    function Layout:on_options() publish_layout() end
    Layout:new()
    publish_layout()
end
