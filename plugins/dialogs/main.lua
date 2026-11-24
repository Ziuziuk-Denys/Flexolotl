local FORGET_AFTER = 300

local spoken, heard_at

local function update()
  for _, message in ipairs(flexolotl.messages()) do
    local said = flexolotl.json(message)
    if type(said) == "table" and said.line then
      spoken, heard_at = said, flexolotl.now()
    end
  end
  if not spoken or flexolotl.now() - heard_at > FORGET_AFTER then
    return nil
  end
  local line = tostring(spoken.line)
  if spoken.speaker then
    line = tostring(spoken.speaker) .. ": " .. line
  end
  return {
    title = tostring(spoken.game or "In a dialog"),
    subtitle = line,
    picture = tostring(spoken.picture or ""),
  }
end

return { every = 1, update = update }
