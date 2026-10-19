-- The autoscaler's policy, one pass per quantum:
--
-- 1. Accept every prospect the fleet can serve now, at most MAX_ACCEPTS per
--    quantum, in snapshot order.
-- 2. When free vCPU or free memory falls under `low_pct` percent of the fleet
--    total, buy one host of `sku`, provided cash stays at or above
--    `cash_reserve` and `cooldown_quanta` quanta have passed since the last
--    purchase.
--
-- A command issued at quantum `q` lands at `q + 2`, and a snapshot reflects it
-- once its own quantum reaches that. Until then a prospect already accepted,
-- and a host already bought, are not ordered again; after it, a prospect still
-- listed is accepted again.
--
-- The mod keeps four things between steps, all in its saved state: the orders
-- in flight, the last purchase quantum, the pause switch and the last ten
-- decisions. A mod rebuilt from that state resumes where it left off. If the
-- saved state has the wrong shape, the mod logs it and starts paused.
--
-- Two panels show the policy and the decisions. The pause toggle on the policy
-- panel stops the mod issuing commands; the mod reads it from the UI input
-- events. It skips any event or UI value of a kind it does not know.
--
-- A config value of nil selects the field's default. An integer field takes 0
-- through math.maxinteger, and so does `cash_reserve` when given an integer.

local Policy = {}
Policy.__index = Policy

--- Most prospects accepted in one quantum.
local MAX_ACCEPTS = 8

--- Quanta between a command's issue and its landing.
local LANDING_QUANTA = 2

--- Decisions kept in state and shown.
local MAX_DECISIONS = 10

local MAX_INT = math.maxinteger

--- The SKUs the `sku` config key names: its enum value and the catalog name.
local SKUS = {
  balanced_x86_1u = "Balanced x86 1U",
  balanced_x86_2u = "Balanced x86 2U",
  arm_1u = "ARM 1U",
  arm_2u = "ARM 2U",
  compute_1u = "Compute 1U",
  compute_2u = "Compute 2U",
  memory_1u = "Memory 1U",
  memory_2u = "Memory 2U",
  storage_1u = "Storage 1U",
  storage_2u = "Storage 2U",
}

local DEFAULTS = {
  sku = "balanced_x86_1u",
  low_pct = 20,
  cash_reserve = 5000.0,
  cooldown_quanta = 8,
}

--- Each key of `config` that is present, over the defaults. `sku` comes out as
--- the catalog name.
local function merge(config)
  local out = {}
  for key, default in pairs(DEFAULTS) do
    local value = config and config[key]
    if value == nil then
      out[key] = default
    elseif key == "low_pct" or key == "cooldown_quanta" then
      if math.type(value) ~= "integer" or value < 0 then
        error("config: " .. key .. " must be a non-negative integer")
      end
      out[key] = value
    elseif type(value) ~= type(default) and not (type(value) == "number" and type(default) == "number") then
      error("config: " .. key .. " has the wrong type")
    elseif key == "cash_reserve" and math.type(value) == "integer" and value < 0 then
      error("config: " .. key .. " must not be a negative integer")
    else
      out[key] = value
    end
  end
  if SKUS[out.sku] == nil then
    error("config: sku is not one of the listed SKUs")
  end
  out.sku = SKUS[out.sku]
  return out
end

--- a * b for non-negative integers, clamped at the largest integer.
local function saturating_mul(a, b)
  if a ~= 0 and b > MAX_INT // a then
    return MAX_INT
  end
  return a * b
end

--- a + b for non-negative integers, clamped at the largest integer.
local function saturating_add(a, b)
  if b > MAX_INT - a then
    return MAX_INT
  end
  return a + b
end

--- True when a resource has a known total and less than `low_pct` percent of
--- it is free.
local function is_low(free, total, low_pct)
  return total > 0 and saturating_mul(free, 100) < saturating_mul(total, low_pct)
end

local function capacity_is_low(c, low_pct)
  return is_low(c.free_vcpu_milli, c.total_vcpu_milli, low_pct)
    or is_low(c.free_mem_mb, c.total_mem_mb, low_pct)
end

local function is_count(v)
  return math.type(v) == "integer" and v >= 0
end

local function is_decision(row)
  return type(row) == "table"
    and type(row.action) == "string"
    and is_count(row.index)
    and is_count(row.quantum)
    and type(row.result) == "string"
    and type(row.target) == "string"
end

local function is_pending(pair)
  return type(pair) == "table" and is_count(pair[1]) and is_count(pair[2])
end

--- True when `list` is nil, or a table whose every entry passes `check`.
local function all_or_nil(list, check)
  if list == nil then
    return true
  end
  if type(list) ~= "table" then
    return false
  end
  for _, item in ipairs(list) do
    if not check(item) then
      return false
    end
  end
  return true
end

--- The name of the first field of a saved state that does not hold its shape,
--- or nil when every one does.
local function shape_error(state)
  if type(state) ~= "table" then
    return "not a table"
  elseif state.last_buy_quantum ~= nil and not is_count(state.last_buy_quantum) then
    return "last_buy_quantum"
  elseif state.paused ~= nil and type(state.paused) ~= "boolean" then
    return "paused"
  elseif not all_or_nil(state.decisions, is_decision) then
    return "decisions"
  elseif not all_or_nil(state.pending, is_pending) then
    return "pending"
  end
  return nil
end

--- The state a mod starts from: the saved one, or an empty one on a first run.
--- A saved state with the wrong shape gives an empty, paused state, and the
--- second result says why.
local function restore(state)
  local out = { decisions = {}, last_buy_quantum = nil, paused = false, pending = {} }
  if state == nil then
    return out
  end
  local bad = shape_error(state)
  if bad ~= nil then
    out.paused = true
    return out, "saved state did not decode (" .. bad .. "); starting paused"
  end
  out.last_buy_quantum = state.last_buy_quantum
  out.paused = state.paused == true
  for _, row in ipairs(state.decisions or {}) do
    out.decisions[#out.decisions + 1] = {
      action = row.action,
      index = row.index,
      quantum = row.quantum,
      result = row.result,
      target = row.target,
    }
  end
  for _, pair in ipairs(state.pending or {}) do
    out.pending[#out.pending + 1] = { pair[1], pair[2] }
  end
  return out
end

---@param config table|nil
---@param state table|nil
function Policy.new(config, state)
  local restored, note = restore(state)
  return setmetatable({
    config = merge(config),
    state = restored,
    state_note = note,
  }, Policy)
end

--- Rewrites a state saved under schema 0, `{last_buy_quantum}`, as schema 1.
---@param from_schema integer
---@param state table
function Policy.migrate(from_schema, state)
  if from_schema == 0 then
    if type(state) ~= "table" or math.type(state.last_buy_quantum) ~= "integer" then
      error("state schema 0 holds no last_buy_quantum")
    end
    return {
      decisions = {},
      last_buy_quantum = state.last_buy_quantum,
      paused = false,
      pending = {},
    }
  end
  error("no migration from state schema " .. tostring(from_schema))
end

--- True while a command issued at `issued` has not reached a snapshot taken at
--- `snapshot_quantum`.
local function in_flight(issued, snapshot_quantum)
  return snapshot_quantum < issued + LANDING_QUANTA
end

--- Fills in the result of the decision a receipt answers.
function Policy:settle(r)
  for _, d in ipairs(self.state.decisions) do
    if d.quantum == r.quantum and d.index == r.index and d.result == "pending" then
      if r.ok then
        d.result = "ok"
      elseif r.code == nil or r.code == "" then
        d.result = "refused"
      else
        d.result = r.code
      end
      return
    end
  end
end

--- Applies the receipts and UI inputs of one batch, in batch order.
function Policy:read_events(events)
  for _, ev in ipairs(events or {}) do
    if ev.Receipt then
      self:settle(ev.Receipt)
    elseif ev.UiInput then
      local input = ev.UiInput
      -- Only the host (player 0) can pause; a guest's press is ignored.
      if
        input.player == 0
        and input.panel == "policy"
        and input.widget == "paused"
        and type(input.value) == "table"
      then
        if type(input.value.Bool) == "boolean" then
          self.state.paused = input.value.Bool
        end
      end
    end
  end
end

function Policy:record(action, quantum, index, target)
  local decisions = self.state.decisions
  decisions[#decisions + 1] = {
    action = action,
    index = index,
    quantum = quantum,
    result = "pending",
    target = target,
  }
  if #decisions > MAX_DECISIONS then
    table.remove(decisions, 1)
  end
end

local function stat(value, detail, tone)
  return { Stat = { value = value, detail = detail, tone = tone } }
end

function Policy:policy_panel(now)
  local c = self.config
  local last = self.state.last_buy_quantum
  local next_value, tone, detail = "now", "Good", "next buy allowed at quantum"
  if last ~= nil then
    local at = saturating_add(last, c.cooldown_quanta)
    next_value = tostring(at)
    tone = now >= at and "Good" or "Info"
  end
  if self.state.paused then
    tone = "Warn"
    detail = "paused: the autoscaler issues nothing"
  end
  return {
    values = {
      sku = stat(c.sku, "host bought", "Neutral"),
      low_pct = stat(tostring(c.low_pct) .. "%", "buy under this free share", "Neutral"),
      cash_reserve = stat(string.format("$%.2f", c.cash_reserve), "cash to keep", "Neutral"),
      cooldown = stat(tostring(c.cooldown_quanta) .. " quanta", "between purchases", "Neutral"),
      next_buy = stat(next_value, detail, tone),
      paused = { Control = { Bool = self.state.paused } },
    },
  }
end

function Policy:decisions_panel()
  local rows = {}
  local decisions = self.state.decisions
  for i = #decisions, 1, -1 do
    local d = decisions[i]
    rows[#rows + 1] = { tostring(d.quantum), d.action, d.target, d.result }
  end
  return { values = { recent = { Rows = rows } } }
end

---@param ctx uptime.Ctx
function Policy:step(ctx)
  local now = ctx.quantum.index
  local snapshot = ctx.snapshot
  local state = self.state

  if self.state_note ~= nil then
    ctx:log(4, self.state_note)
    self.state_note = nil
  end

  self:read_events(ctx.events)

  -- The snapshot refreshes every few quanta and the quanta between see the
  -- same prospects and the same shortage. An order is held back until a
  -- snapshot can show it landed.
  local seen = snapshot.quantum
  local kept = {}
  local held = {}
  for _, pair in ipairs(state.pending) do
    if in_flight(pair[2], seen) then
      kept[#kept + 1] = pair
      held[pair[1]] = true
    end
  end
  state.pending = kept

  if not state.paused then
    local fresh = {}
    for _, p in ipairs(snapshot.prospects or {}) do
      if p.fit_can_serve_now and not p.fit_over_capacity and not held[p.id] then
        fresh[#fresh + 1] = p.id
        if #fresh == MAX_ACCEPTS then
          break
        end
      end
    end
    for _, id in ipairs(fresh) do
      local index = ctx:issue({ AcceptProspect = { prospect_id = id } })
      state.pending[#state.pending + 1] = { id, now }
      self:record("accept", now, index, "prospect " .. tostring(id))
    end
    table.sort(state.pending, function(a, b)
      return a[1] < b[1]
    end)

    local cfg = self.config
    local last = state.last_buy_quantum
    local cooled = last == nil or math.max(now - last, 0) >= cfg.cooldown_quanta
    local affordable = snapshot.topbar ~= nil and snapshot.topbar.cash_dollars >= cfg.cash_reserve
    local low = snapshot.capacity ~= nil and capacity_is_low(snapshot.capacity, cfg.low_pct)
    local landed = last == nil or not in_flight(last, seen)
    if cooled and landed and affordable and low then
      local index = ctx:issue({ BuyHost = { sku_name = cfg.sku } })
      state.last_buy_quantum = now
      self:record("buy", now, index, cfg.sku)
      ctx:log(2, "capacity low: buying a host")
    end
  end

  ctx:save_state({
    decisions = state.decisions,
    last_buy_quantum = state.last_buy_quantum,
    paused = state.paused,
    pending = state.pending,
  })
  ctx:panel("policy", self:policy_panel(now))
  ctx:panel("decisions", self:decisions_panel())
end

return Policy
