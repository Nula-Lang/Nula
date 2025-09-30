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

        // Validate file extension (optional, but recommended)
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
                // Validate target (example architectures)
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

        // Read input file
        const file = try std.fs.cwd().openFile(file_path, .{});
        defer file.close();
        const content = try file.readToEndAlloc(allocator, 10 * 1024 * 1024); // 10MB limit
        defer allocator.free(content);

        // Apply optimization passes
        var optimized = try pass_remove_nops(allocator, content);
        defer allocator.free(optimized);

        var next_optimized = try pass_constant_folding(allocator, optimized);
        allocator.free(optimized); // Free previous buffer
        optimized = next_optimized;

        if (release) {
            next_optimized = try pass_dead_code_elim(allocator, optimized);
            allocator.free(optimized);
            optimized = next_optimized;
        }

        // Handle target-specific optimizations
        if (target) |t| {
            std.debug.print("Applying target-specific optimizations for {s}\n", .{t});
            // Placeholder for target-specific optimizations
        }

        // Generate output file path
        const out_path = if (release) blk: {
            const ext = std.fs.path.extension(file_path);
            const stem = std.fs.path.stem(file_path);
            break :blk try std.fmt.allocPrint(allocator, "{s}.opt{s}", .{ stem, ext });
        } else file_path;
        defer if (release) allocator.free(out_path);

        // Write optimized assembly
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
        if (std.mem.eql(u8, trimmed, "nop")) continue;
        try list.appendSlice(line);
        try list.append('\n');
    }
    return list.toOwnedSlice();
}

fn pass_constant_folding(allocator: std.mem.Allocator, input: []const u8) ![]u8 {
    var list = std.ArrayList(u8).init(allocator);
    defer list.deinit();

    var lines = std.mem.splitSequence(u8, input, "\n");
    var prev: ?[]const u8 = null;
    while (lines.next()) |line| {
        if (prev) |p| {
            const trimmed_prev = std.mem.trim(u8, p, " \t");
            const trimmed_line = std.mem.trim(u8, line, " \t");
            if (std.mem.startsWith(u8, trimmed_prev, "mov $") and std.mem.startsWith(u8, trimmed_line, "add $")) {
                // Extract values from mov and add instructions
                var mov_parts = std.mem.splitSequence(u8, trimmed_prev, ",");
                var add_parts = std.mem.splitSequence(u8, trimmed_line, ",");

                // Get the first part of mov and add (the value after the instruction)
                const mov_first = mov_parts.next() orelse continue;
                const add_first = add_parts.next() orelse continue;
                const mov_val_str = std.mem.trim(u8, mov_first[4..], " \t"); // Skip "mov $"
                const add_val_str = std.mem.trim(u8, add_first[4..], " \t"); // Skip "add $"

                // Get the register (second part)
                const mov_reg = std.mem.trim(u8, mov_parts.next() orelse continue, " \t");
                const add_reg = std.mem.trim(u8, add_parts.next() orelse continue, " \t");

                if (std.mem.eql(u8, mov_reg, add_reg)) {
                    // Same register, attempt to fold
                    const mov_val = std.fmt.parseInt(i32, mov_val_str, 10) catch continue;
                    const add_val = std.fmt.parseInt(i32, add_val_str, 10) catch continue;
                    const folded = try std.fmt.allocPrint(allocator, "mov ${d}, {s}\n", .{ mov_val + add_val, add_reg });
                    defer allocator.free(folded);
                    try list.appendSlice(folded);
                    prev = null;
                    continue;
                }
            }
            try list.appendSlice(p);
            try list.append('\n');
        }
        prev = line;
    }
    if (prev) |p| {
        try list.appendSlice(p);
        try list.append('\n');
    }
    return list.toOwnedSlice();
}

fn pass_dead_code_elim(allocator: std.mem.Allocator, input: []const u8) ![]u8 {
    // Placeholder for dead code elimination
    return try allocator.dupe(u8, input);
}

