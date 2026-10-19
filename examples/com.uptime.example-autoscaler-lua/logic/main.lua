-- Example autoscaler (com.uptime.example-autoscaler-lua)
--
-- Entry point. It builds the policy from the mod's config and saved state and
-- passes each step to it. The policy itself is in policy.lua.

local policy = require("policy")

local M = {}

local autoscaler

---@param ctx uptime.InitCtx
function M.init(ctx)
  autoscaler = policy.new(ctx.config, ctx.state)
end

---@param ctx uptime.Ctx
function M.step(ctx)
  autoscaler:step(ctx)
end

---@param from_schema integer
---@param state table
function M.migrate(from_schema, state)
  return policy.migrate(from_schema, state)
end

return M
