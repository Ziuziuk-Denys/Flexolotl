# Plugins

A plugin is a source of statuses. Flexolotl asks it every few seconds what to show, the plugin answers. Install one in the app: *Settings, Plugins*.

## Write one

A plugin is a folder with two files. Put it into the folder of plugins (*Settings, Plugins, Open the folder of plugins*) and it appears in the app.

```
my-plugin/
  plugin.json
  main.lua
```

`plugin.json`

```json
{
  "id": "my-plugin",
  "name": "My plugin",
  "version": "1.0.0",
  "api": 1,
  "author": "You",
  "about": "What it shows, in one line",
  "permissions": { "hosts": ["api.example.com"] }
}
```

Optional: `"translations": { "ru": { "name": "", "about": "" } }`.

`main.lua`

```lua
local function update()
  return { title = "Reading", subtitle = "chapter 5" }
end

return { every = 10, update = update }
```

`every` is how often to ask, in seconds, from 1 to 300. `update` returns what to show or `nil` for nothing:

| Field | |
| - | - |
| `title` | the first line, required |
| `subtitle` | the second line |
| `kind` | `"playing"`, `"listening"`, `"watching"` or `"competing"` |
| `picture` | an `https` link to a picture or a GIF |
| `started`, `ends` | seconds since 1970, for a timer |
| `buttons` | up to two `{ label = "", link = "https://" }` |

## What a plugin can use

Plugins run in Lua 5.4 without access to the computer: there is no `io`, `os`, `require` or `pcall`, 24 MB of memory and 3 seconds for one update. Everything else is asked for in `permissions` and shown to the person before the plugin runs. When the permissions change, the person is asked again.

| Permission | Gives |
| - | - |
| `"files": ["~/.game/log.txt"]` | `flexolotl.follow(path)`: lines added since the last call. `flexolotl.read(path)`: the text, up to 1 MB, or `nil`. A path that ends with `/` allows the whole folder. Starts with `~/`, `%APPDATA%/`, `%LOCALAPPDATA%/` or `/` |
| `"hosts": ["api.example.com"]` | `flexolotl.get(url)`: the answer of an `https` address, or `nil`. Redirects are not followed |
| `"port": 4177` | `flexolotl.messages()`: what games and mods sent to this port since the last call |
| `"programs": true` | `flexolotl.running(name)`: whether a program is running |

Without any permission: `flexolotl.now()`, the time in seconds, and `flexolotl.json(text)`, a table or `nil`.

## Talk to a plugin from a game

A plugin with a port takes `POST` requests on the same computer. The header `X-Flexolotl` is required, so web pages cannot send anything.

```sh
curl -H "X-Flexolotl: 1" -d '{"game":"Deep Cave","speaker":"Mira","line":"Do not look down"}' http://127.0.0.1:4177
```

## Share it

Add a folder here in a pull request, run `UPDATE_INDEX=1 cargo test -p flexolotl-plugins --test catalog`, commit. Plugins are read before they are accepted: ask only for what you need.
