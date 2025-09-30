const std = @import("std");

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const args = try std.process.argsAlloc(allocator);
    defer std.process.argsFree(allocator, args);

    if (args.len < 2) {
        std.debug.print(
            \\Usage: nula-zig <command> [args]
            \\Commands:
            \\  optimize <asm_file> [--release] [--target <arch>]
            \\
            , .{});
        return error.MissingCommand;
    }

    const cmd = args[1];
    if (std.mem.eql(u8, cmd, "optimize")) {
        if (args.len < 3) {
            std.debug.print("Error: Missing asm_file argument\n", .{});
            return error.MissingArgument;
        }
        const file_path = args[2];

        if (!std.mem.endsWith(u8, file_path, ".s")) {
            std.debug.print("Warning: Input file '{s}' does not have .s extension\n", .{file_path});
        }

        var release = false;
        var target: ?[]const u8 = null;
        var i: usize = 3;
        while (i < args.len) : (i += 1) {
            if (std.mem.eql(u8, args[i], "--release")) {
                release = true;
            } else if (std.mem.eql(u8, args[i], "--target")) {
                if (i + 1 >= args.len) {
                    std.debug.print("Error: --target requires an architecture\n", .{});
                    return error.InvalidArgument;
                }
                target = args[i + 1];
                i += 1;
                const valid_targets = [_][]const u8{ "x86_64", "aarch64", "riscv64" };
                var is_valid = false;
                for (valid_targets) |valid_target| {
                    if (std.mem.eql(u8, target.?, valid_target)) {
                        is_valid = true;
                        break;
                    }
                }
                if (!is_valid) {
                    std.debug.print("Error: Invalid target '{s}'. Supported: x86_64, aarch64, riscv64\n", .{target.?});
                    return error.InvalidTarget;
                }
            } else {
                std.debug.print("Warning: Unknown argument '{s}'\n", .{args[i]});
            }
        }

        const file = try std.fs.cwd().openFile(file_path, .{});
        defer file.close();
        const content = try file.readToEndAlloc(allocator, 10 * 1024 * 1024); // 10MB limit
        defer allocator.free(content);

        var optimized = try pass_remove_nops(allocator, content);
        defer allocator.free(optimized);

        var next_optimized = try pass_constant_folding(allocator, optimized);
        allocator.free(optimized);
        optimized = next_optimized;

        next_optimized = try pass_dead_code_elim(allocator, optimized);
        allocator.free(optimized);
        optimized = next_optimized;

        next_optimized = try pass_redundant_load_elim(allocator, optimized);
        allocator.free(optimized);
        optimized = next_optimized;

        if (release) {
            next_optimized = try pass_aggressive_optim(allocator, optimized);
            allocator.free(optimized);
            optimized = next_optimized;
        }

        if (target) |t| {
            std.debug.print("Applying target-specific optimizations for {s}\n", .{t});
            optimized = switch_target_optim(allocator, optimized, t) catch optimized;
        }

        const out_path = if (release) blk: {
            const dir = std.fs.path.dirname(file_path) orelse ".";
            const stem = std.fs.path.stem(file_path);
            const ext = std.fs.path.extension(file_path);
            const new_name = try std.fmt.allocPrint(allocator, "{s}.opt{s}", .{ stem, ext });
            defer allocator.free(new_name);
            break :blk try std.fs.path.join(allocator, &[_][]const u8{ dir, new_name });
        } else file_path;
        defer if (release) allocator.free(out_path);

        try std.fs.cwd().writeFile(.{ .sub_path = out_path, .data = optimized });

        std.debug.print("Optimized {s} to {s} (release: {}, target: {?s})\n", .{ file_path, out_path, release, target });
    } else {
        std.debug.print("Error: Unknown command '{s}'\n", .{cmd});
        return error.UnknownCommand;
    }
}

fn pass_remove_nops(allocator: std.mem.Allocator, input: []const u8) ![]u8 {
    var list = std.ArrayList(u8).init(allocator);
    defer list.deinit();

    var lines = std.mem.splitSequence(u8, input, "\n");
    while (lines.next()) |line| {
        const trimmed = std.mem.trim(u8, line, " \t");
        if (std.mem.eql(u8, trimmed, "nop") or std.mem.startsWith(u8, trimmed, ";") or trimmed.len == 0) continue;
        try list.appendSlice(line);
        try list.append('\n');
    }
    return list.toOwnedSlice();
}

fn pass_constant_folding(allocator: std.mem.Allocator, input: []const u8) ![]u8 {
    var list = std.ArrayList(u8).init(allocator);
    defer list.deinit();

    var lines_iter = std.mem.splitSequence(u8, input, "\n");
    var lines = std.ArrayList([]const u8).init(allocator);
    defer lines.deinit();
    while (lines_iter.next()) |line| {
        try lines.append(line);
    }

    var idx: usize = 0;
    while (idx < lines.items.len) : (idx += 1) {
        const line = lines.items[idx];
        const trimmed = std.mem.trim(u8, line, " \t");
        if (std.mem.startsWith(u8, trimmed, "mov $") and idx + 1 < lines.items.len) {
            const next_line = std.mem.trim(u8, lines.items[idx + 1], " \t");
            if (std.mem.startsWith(u8, next_line, "add $")) {
                var mov_iter = std.mem.splitScalar(u8, trimmed[4..], ',');
                const mov_val_str = std.mem.trim(u8, mov_iter.next() orelse continue, " \t");
                const mov_reg = std.mem.trim(u8, mov_iter.next() orelse continue, " \t");

                var add_iter = std.mem.splitScalar(u8, next_line[4..], ',');
                const add_val_str = std.mem.trim(u8, add_iter.next() orelse continue, " \t");
                const add_reg = std.mem.trim(u8, add_iter.next() orelse continue, " \t");

                if (std.mem.eql(u8, mov_reg, add_reg)) {
                    const mov_val = std.fmt.parseInt(i64, mov_val_str, 0) catch continue;
                    const add_val = std.fmt.parseInt(i64, add_val_str, 0) catch continue;
                    const folded = try std.fmt.allocPrint(allocator, "    mov ${d}, {s}\n", .{ mov_val + add_val, add_reg });
                    defer allocator.free(folded);
                    try list.appendSlice(folded);
                    idx += 1; // Skip next line
                    continue;
                }
            }
        }
        try list.appendSlice(line);
        try list.append('\n');
    }
    return list.toOwnedSlice();
}

fn pass_dead_code_elim(allocator: std.mem.Allocator, input: []const u8) ![]u8 {
    var list = std.ArrayList(u8).init(allocator);
    defer list.deinit();

    var lines = std.mem.splitSequence(u8, input, "\n");
    var in_dead = false;
    while (lines.next()) |line| {
        const trimmed = std.mem.trim(u8, line, " \t");
        if (std.mem.startsWith(u8, trimmed, "jmp")) {
            in_dead = true;
        } else if (std.mem.startsWith(u8, trimmed, ".L")) { // Label
            in_dead = false;
        }
        if (!in_dead) {
            try list.appendSlice(line);
            try list.append('\n');
        }
    }
    return list.toOwnedSlice();
}

fn pass_redundant_load_elim(allocator: std.mem.Allocator, input: []const u8) ![]u8 {
    var list = std.ArrayList(u8).init(allocator);
    defer list.deinit();

    var lines = std.mem.splitSequence(u8, input, "\n");
    var prev_reg: ?[]const u8 = null;
    var prev_val: ?[]const u8 = null;
    while (lines.next()) |line| {
        const trimmed = std.mem.trim(u8, line, " \t");
        if (std.mem.startsWith(u8, trimmed, "mov ")) {
            var iter = std.mem.splitScalar(u8, trimmed[4..], ',');
            const val = std.mem.trim(u8, iter.next() orelse continue, " \t");
            const reg = std.mem.trim(u8, iter.next() orelse continue, " \t");
            if (prev_reg) |pr| {
                if (std.mem.eql(u8, pr, reg) and std.mem.eql(u8, prev_val.?, val)) {
                    continue; // Skip redundant mov
                }
            }
            prev_reg = reg;
            prev_val = val;
        } else {
            prev_reg = null;
            prev_val = null;
        }
        try list.appendSlice(line);
        try list.append('\n');
    }
    return list.toOwnedSlice();
}

fn pass_aggressive_optim(allocator: std.mem.Allocator, input: []const u8) ![]u8 {
    // More optimizations, e.g., loop unrolling, inlining, etc.
    // For now, duplicate constant folding as example
    return pass_constant_folding(allocator, input);
}

fn switch_target_optim(allocator: std.mem.Allocator, input: []const u8, target: []const u8) ![]u8 {
    // Target specific
    if (std.mem.eql(u8, target, "aarch64")) {
        // Convert x86 to arm instructions, but placeholder
    }
    return try allocator.dupe(u8, input);
}
