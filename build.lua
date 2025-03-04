local run = function(cmd)
	local handle = io.popen(cmd)
	if not handle then
		return "failed to cd"
	end
	if type(handle) == "string" then
		return handle
	end
	local output = handle:read("*a")
	handle:close()
	local i = 0
	-- Below taken from <https://www.lua.org/pil/20.1.html>, with some modification
	while true do
		local next = string.find(output, "\n", i + 1) -- find 'next' newline
		if next == nil then
			break
		end
		i = next
	end
	local exit = string.sub(output, i - 1)
	-- Why can't I trim whitespace?
	local code = string.sub(exit, 0, 1)
	if code == "0" then
		return nil
	end
	return code
end
local dir = vim.fn.fnamemodify(debug.getinfo(1, "S").source:sub(2), ":p:h")
-- Have to echo $? so that I can parse out the exit code
local cmd = string.format(
	"sh -c 'cd %s/lua/nvim_winpick && cargo b -r -p nvim_winpick &> /dev/null && cp target/release/libnvim_winpick.so ../nvim_winpick.so; echo $?'",
	dir
)
local run_err = run(cmd)
if run_err then
	error(
		string.format(
			"build exited with error %s, see https://github.com/MarcusGrass/nvim_winpick/blob/main/Readme.md for build instructions",
			run_err
		),
		3
	)
end
