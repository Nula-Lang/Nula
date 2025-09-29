const std = @import("std");

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const args = std.process.argsAlloc(allocator) catch return;
    defer std.process.argsFree(allocator, args);

    if (args.len < 2) {
        std.debug.print("Usage: nula-zig <file.s>\n", .{});
        return;
    }

    const input_file = std.fs.cwd().openFile(args[1], .{}) catch |err| {
        std.debug.print("Error opening {s}: {}\n", .{ args[1], err });
        return;
    };
    defer input_file.close();

    var buf: [1024]u8 = undefined;
    const content = input_file.readAll(&buf, allocator) catch return;

    // Prosta optymalizacja: usuń niepotrzebne mov, dodaj inline asm dla wydajności blisko C
    var optimized = std.ArrayList(u8).init(allocator);
    defer optimized.deinit();

    // Przykład: dodaj .section .text; optimized.appendSlice(content) ...
    optimized.writer().print(".section .text\n") catch return;
    optimized.appendSlice(content) catch return;  // Tutaj dodaj opt: np. peephole

    const out_file = std.fs.cwd().createFile("optimized.s", .{}) catch return;
    defer out_file.close();
    out_file.writeAll(optimized.items) catch return;

    std.debug.print("Optimized to optimized.s\n", .{});
}
