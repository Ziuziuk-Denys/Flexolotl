local logs = {
  "~/.minecraft/logs/latest.log",
  "%APPDATA%/.minecraft/logs/latest.log",
  "~/Library/Application Support/minecraft/logs/latest.log",
}

local place, since, advancement

local function is_address(host)
  return host:match("^%d+%.%d+%.%d+%.%d+$") or host == "localhost"
end

local function enter(where)
  place, since, advancement = where, flexolotl.now(), nil
end

local function read(line)
  local host = line:match("Connecting to ([^,]+),")
  if host then
    enter(is_address(host) and "Playing with friends" or "On " .. host)
  elseif line:find("Starting integrated minecraft server", 1, true) then
    enter("In a world of their own")
  elseif line:find("Stopping!", 1, true) or line:find("Disconnecting from server", 1, true) then
    place = nil
  else
    advancement = line:match("%[CHAT%].+ has %a+ the %a+ %[(.-)%]") or advancement
  end
end

local function update()
  for _, log in ipairs(logs) do
    for _, line in ipairs(flexolotl.follow(log)) do
      read(line)
    end
  end
  if not place then
    return nil
  end
  return {
    title = "Minecraft",
    subtitle = advancement and place .. ", got " .. advancement or place,
    started = since,
  }
end

return { every = 3, update = update }
