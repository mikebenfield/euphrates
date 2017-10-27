
function step()
    command("Step")
end

function hold()
    command("Hold")
end

function resume()
    command("Resume")
end

function breakpoint(pc)
    command({"BreakAtPc", pc})
end

function remove_breakpoints()
    command("RemovePcBreakpoints")
end

function remove_breakpoint_messages()
    command("RemoveBreakMessages")
end

function break_at_message(x)
    command({"BreakAtMessage", x})
end

function messages()
    command("ShowRecentMessages")
end

function z80_status()
    command("Z80Status")
end

function disassemble()
    command("Disassemble")
end
