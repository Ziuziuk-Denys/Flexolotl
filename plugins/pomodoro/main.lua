local WORK, REST = 25 * 60, 5 * 60

local start = flexolotl.now()

local function update()
  local passed = (flexolotl.now() - start) % (WORK + REST)
  local round = start + math.floor((flexolotl.now() - start) / (WORK + REST)) * (WORK + REST)
  if passed < WORK then
    return { title = "Focusing", subtitle = "back soon", ends = round + WORK }
  end
  return { title = "On a break", subtitle = "write now", ends = round + WORK + REST }
end

return { every = 5, update = update }
