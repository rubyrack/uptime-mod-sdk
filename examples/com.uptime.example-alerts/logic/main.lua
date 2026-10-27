-- Example alerts (com.uptime.example-alerts)
--
-- Shows the three mod UI hooks and issues no command, so it never affects a
-- run's achievements:
--   * a HUD card that counts the open incidents,
--   * a toast when a DDoS attack starts,
--   * a dashboard panel that opens on F6, whose Acknowledge button answers F7
--     from anywhere and quiets the card until the count changes.

local M = {}

-- Open incidents at which the card turns from a warning into a danger.
local DANGER_AT = 3

local state

---@param ctx uptime.InitCtx
function M.init(ctx)
  local saved = ctx.state or {}
  state = {
    acked = saved.acked or 0,
    attacks = saved.attacks or 0,
    last = saved.last or "None yet",
  }
end

-- The tone and detail of the HUD card for `open` incidents.
local function hud_stat(open)
  if open == 0 then
    return { value = "0", detail = "all clear", tone = "Good" }
  end
  if open == state.acked then
    return { value = tostring(open), detail = "acknowledged", tone = "Neutral" }
  end
  local tone = open >= DANGER_AT and "Danger" or "Warn"
  return { value = tostring(open), detail = "open the board", tone = tone }
end

---@param ctx uptime.Ctx
function M.step(ctx)
  local open = #(ctx.snapshot.active_incidents or {})

  local started = 0
  for _, ev in ipairs(ctx.events) do
    local engine = ev.Engine and ev.Engine.event
    if type(engine) == "table" and engine.DdosStarted then
      started = started + 1
      state.attacks = state.attacks + 1
      state.last = string.format(
        "%d Gbps for %d h", engine.DdosStarted.gbps, engine.DdosStarted.hours
      )
    end
    local input = ev.UiInput
    if input and input.panel == "board" and input.widget == "ack" then
      state.acked = open
    end
  end
  if open == 0 then
    state.acked = 0
  end

  -- One toast per step, so a burst of attacks stays under the cap of two.
  if started > 0 then
    local text = started == 1 and ("DDoS attack: " .. state.last)
      or string.format("%d DDoS attacks started", started)
    ctx:toast("Danger", text)
  end

  -- An empty update clears the card, so it shows only while an incident is open.
  ctx:panel("hud", { values = open == 0 and {} or { open = { Stat = hud_stat(open) } } })
  ctx:panel("board", {
    values = {
      open = { Stat = hud_stat(open) },
      attacks = { Stat = { value = tostring(state.attacks), detail = "", tone = "Neutral" } },
      last = { Text = state.last },
    },
  })

  ctx:save_state(state)
end

---@param _from_schema integer
---@param saved table
function M.migrate(_from_schema, saved)
  return saved
end

return M
