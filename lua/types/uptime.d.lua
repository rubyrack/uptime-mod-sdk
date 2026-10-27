---@meta

-- The Lua mod API: the shape of the entry chunk, the `ctx` a step receives and
-- the `uptime` global. Hand-written. The snapshot, command and event types it
-- names are generated into the sibling `uptime-*.d.lua` files.

---@class uptime.Quantum
---@field index integer
---@field sim_time_ns integer
---@field seed integer

--- A log level: `uptime.LOG.trace` (0) to `uptime.LOG.error` (4).
---@alias uptime.LogLevel 0|1|2|3|4

--- The context `M.init` receives.
---@class uptime.InitCtx
---@field config table
---@field state table|nil

--- The context `M.step` receives.
---@class uptime.Ctx
---@field quantum uptime.Quantum
---@field snapshot uptime.snapshot.ModSnapshot
---@field events uptime.ModEvent[]
local Ctx = {}

--- Queues a command. It lands two quanta later. Returns its index, which a
--- later receipt carries.
---@param cmd uptime.Command
---@return integer
function Ctx:issue(cmd) end

--- Saves the mod's state. The last call in a step wins.
---@param tbl table
function Ctx:save_state(tbl) end

--- Sets the values of panel `id`, a panel the pack's panels file declares.
--- They land with this step's commands, two quanta later. The last call for
--- an `id` in a step wins. An empty `values` clears the panel, which hides a
--- HUD card.
---@param id string
---@param update uptime.PanelUpdate
function Ctx:panel(id, update) end

--- Raises a toast: a short notice every player sees for a few seconds. It
--- lands with this step's commands, two quanta later. At most 2 per step;
--- `text` is 1 to 160 bytes of UTF-8. The game picks the colour from `tone`.
---@param tone uptime.Tone
---@param text string
function Ctx:toast(tone, text) end

--- Writes a log line.
---@param level uptime.LogLevel
---@param msg string
function Ctx:log(level, msg) end

--- The number of commands queued so far.
---@return integer
function Ctx:queued() end

--- The table the entry chunk returns.
---@class uptime.Mod
---@field init? fun(ctx: uptime.InitCtx)
---@field step fun(ctx: uptime.Ctx)
---@field migrate? fun(from_schema: integer, state: table): table

---@class uptime.Log
---@field trace integer
---@field debug integer
---@field info integer
---@field warn integer
---@field error integer

--- The `uptime` global.
---@class uptime.Runtime
---@field LOG uptime.Log
---@field GUEST string
local Runtime = {}

--- Marks a table as an array, so it saves as an array even when it is empty.
---@param t table
---@return table
function Runtime.array(t) end

---@type uptime.Runtime
uptime = Runtime
