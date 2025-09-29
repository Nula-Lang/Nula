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
        return;
    }

    const cmd = args[1];
    if (std.mem.eql(u8, cmd, "optimize")) {
        if (args.len < 3) {
            std.debug.print("Missing asm_file\n", .{});
            return;
        }
        const file_path = args[2];
        var release = false;
        var target: ?[]const u8 = null;
        var i: usize = 3;
        while (i < args.len) : (i += 1) {
            if (std.mem.eql(u8, args[i], "--release")) {
                release = true;
            } else if (std.mem.eql(u8, args[i], "--target") and i + 1 < args.len) {
                target = args[i + 1];
                i += 1;
            }
        }

        const file = try std.fs.cwd().openFile(file_path, .{});
        defer file.close();
        const content = try file.readToEndAlloc(allocator, 1024 * 1024 * 10);
        defer allocator.free(content);

        // Multiple optimization passes
        var optimized = try pass_remove_nops(allocator, content);
        defer allocator.free(optimized);
        optimized = try pass_constant_folding(allocator, optimized);
        defer allocator.free(optimized);
        if (release) {
            optimized = try pass_dead_code_elim(allocator, optimized);
            defer allocator.free(optimized);
        }

        // Handle target-specific
        if (target) |t| {
            std.debug.print("Applying target optimizations for {s}\n", .{t});
            // Placeholder for target opts
        }

        // Write optimized ASM
        const out_path = if (release) try std.fmt.allocPrint(allocator, "{s}.opt", .{file_path}) else file_path;
        defer if (release) allocator.free(out_path);
        try std.fs.cwd().writeFile(.{ .sub_path = out_path, .data = optimized });

        std.debug.print("Optimized {s} (release: {}, target: {?s})\n", .{file_path, release, target});
    } else {
        std.debug.print("Unknown command: {s}\n", .{cmd});
    }
}

fn pass_remove_nops(allocator: std.mem.Allocator, asm: []const u8) ![]u8 {
    var list = std.ArrayList(u8).init(allocator);
    defer list.deinit();

    var lines = std.mem.split(u8, asm, "\n");
    while (lines.next()) |line| {
        if (std.mem.containsAtLeast(u8, line, 1, "nop")) continue;
        try list.appendSlice(line);
        try list.append('\n');
    }
    return list.toOwnedSlice();
}

fn pass_constant_folding(allocator: std.mem.Allocator, asm: []const u8) ![]u8 {
    // Simple folding: mov $1, %rax; add $2, %rax -> mov $3, %rax
    var list = std.ArrayList(u8).init(allocator);
    defer list.deinit();

    var lines = std.mem.split(u8, asm, "\n");
    var prev: ?[]const u8 = null;
    while (lines.next()) |line| {
        if (prev) |p| {
            if (std.mem.containsAtLeast(u8, p, 1, "mov $") and std.mem.containsAtLeast(u8, line, 1, "add $")) {
                // Fold
                const mov_val = std.mem.trimLeft(u8, p["mov $".len..], " ");
                const add_val = std.mem.trimLeft(u8, line["add $".len..], " ");
                const folded = try std.fmt.allocPrint(allocator, "mov ${d}, %rax\n", .{try std.fmt.parseInt(i32, mov_val, 10) + try std.fmt.parseInt(i32, add_val, 10)});
                defer allocator.free(folded);
                try list.appendSlice(folded);
                prev = null;
                continue;
            } else {
                try list.appendSlice(p);
                try list.append('\n');
            }
        }
        prev = line;
    }
    if (prev) |p| {
        try list.appendSlice(p);
        try list.append('\n');
    }
    return list.toOwnedSlice();
}

fn pass_dead_code_elim(allocator: std.mem.Allocator, asm: []const u8) ![]u8 {
    // Placeholder for dead code elimination
    return try allocator.dupe(u8, asm);
}
