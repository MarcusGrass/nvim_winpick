local dir = vim.fn.fnamemodify(debug.getinfo(1, "S").source:sub(2), ":p:h")

local function run(cmd)
	local command = table.concat(cmd, " ")
	local ok, process = pcall(vim.system, cmd, { cwd = dir, text = true })
	if not ok then
		error(string.format("Failed to start build command: %s\nDirectory: %s\n%s", command, dir, process), 0)
	end

	local result = process:wait()
	if result.code ~= 0 or result.signal ~= 0 then
		error(
			string.format(
				"Build command failed (exit %d, signal %d): %s\nDirectory: %s\nstdout:\n%s\nstderr:\n%s",
				result.code,
				result.signal,
				command,
				dir,
				result.stdout or "",
				result.stderr or ""
			),
			0
		)
	end
end

run({ "cargo", "build", "--release", "-p", "nvim_winpick" })
run({ "cp", "target/release/libnvim_winpick.so", "lua/nvim_winpick.so" })
