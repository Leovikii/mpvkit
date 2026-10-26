
-- MpvKit integration, 2026-10-26, GPL-3.0-or-later.
-- Appended to verified uosc 5.13.0; upstream files remain byte-for-byte unchanged.
do
    local function publish_layout()
        local unit = math.max(1, options.controls_size * state.scale)
        local width = math.max(0, display.width - 2 * options.controls_margin * state.scale) / unit
        mp.set_property_native('user-data/mpvkit-uosc', {
            shuffle=state.shuffle, info=width >= 16, tracks=width >= 16,
            queue=width >= 12, speed=width >= 8.5,
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

    -- Style only the configured transport and managed speed button; native hit testing,
    -- delayed click dispatch, badges, menus and navigation remain upstream's responsibility.
    local Controls = require('elements/Controls')
    local function style_controls(controls)
        for _, control in ipairs(controls.controls) do
            local element = control.element
            if element and not element.mpvkit_styled then
                element.mpvkit_styled = true
                local primary = element.prop == 'pause'
                local speed = element.name == 'mpvkit-speed'
                if primary then control.scale = 48 / 44 end
                if speed then control.ratio = 1.5 end
                local on_coordinates = element.on_coordinates
                element.on_coordinates = function(self)
                    on_coordinates(self)
                    local transport = self.icon == 'skip_previous' or self.icon == 'skip_next'
                    self.font_size = round((self.by - self.ay) * (primary and 0.7 or transport and 0.68 or 0.55))
                end
                local render = element.render
                element.render = function(self)
                    local visibility = self:get_visibility()
                    if visibility <= 0 then return end
                    local active = self.active
                    self.active = false -- Status is in the icon/menu, not a persistent filled box.
                    if speed then
                        cursor:zone('primary_click', self, function() self:handle_cursor_click() end)
                        local ass = assdraw.ass_new()
                        if self.proximity_raw <= 0 then
                            ass:rect(self.ax, self.ay, self.bx, self.by, {color=fg, radius=state.radius, opacity=visibility * 0.18})
                            if self.tooltip then ass:tooltip(self, self.tooltip) end
                        end
                        local value = string.format('%g×', mp.get_property_number('speed', 1))
                        ass:txt((self.ax + self.bx) / 2, (self.ay + self.by) / 2, 5, value,
                            {size=round(options.controls_size * state.scale * 0.36), color=fg,
                             border=options.text_border * state.scale, border_color=bg, opacity=visibility})
                        self.active = active
                        return ass
                    end
                    local proximity = self.proximity_raw
                    self.proximity_raw = math.huge -- Replace the upstream 30% white hover fill below.
                    local ass = render(self)
                    self.proximity_raw = proximity
                    self.active = active
                    if (primary or proximity <= 0) and ass then
                        local underlay = assdraw.ass_new()
                        underlay:rect(self.ax, self.ay, self.bx, self.by,
                            {color=fg, radius=state.radius, opacity=visibility * (proximity <= 0 and 0.18 or 0.12)})
                        underlay:new_event()
                        underlay:append(ass.text)
                        if proximity <= 0 and self.tooltip then underlay:tooltip(self, self.tooltip) end
                        return underlay
                    end
                    return ass
                end
            end
        end
        controls:update_dimensions()
    end
    local init_options = Controls.init_options
    function Controls:init_options()
        init_options(self)
        style_controls(self)
    end
    if Elements.controls then style_controls(Elements.controls) end

    local Element = require('elements/Element')
    local Layout = class(Element)
    function Layout:init() Element.init(self, 'mpvkit_layout', {render_order=1.5}) end
    function Layout:on_display() publish_layout() end
    function Layout:on_options() publish_layout() end
    function Layout:render()
        local controls = Elements.controls
        if not controls or not controls.enabled then return end
        local visibility = math.max(0, controls:get_visibility(), Elements:maybe('timeline', 'get_visibility') or 0)
        if visibility <= 0 then return end
        local ass = assdraw.ass_new()
        local top = controls.ay - 4 * state.scale
        local fade = 24 * state.scale
        -- A bounded 24-strip fade avoids adding a separate overlay or gradient dependency.
        for i = 1, 24 do
            ass:rect(0, top - fade + (i - 1) * fade / 24, display.width, top - fade + i * fade / 24,
                {color=bg, opacity=visibility * 0.55 * (i - 0.5) / 24})
        end
        ass:rect(0, top, display.width, display.height, {color=bg, opacity=visibility * 0.55})
        return ass
    end
    Layout:new()
    publish_layout()
end
