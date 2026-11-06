// Windows process containment for browser commands. No terminal engine code.
using System;
using System.ComponentModel;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Text;
using Microsoft.Win32.SafeHandles;

public static class XtermOwnedJob
{
    private const uint KillOnJobClose = 0x00002000;
    private const int ExtendedLimitInformation = 9;
    // The handle belongs to this short-lived wrapper process. Keeping it rooted
    // until Environment.Exit lets Windows close it AFTER preserving our exit code.
    // Closing it earlier would also terminate this wrapper, which is in the job.
    private static SafeFileHandle job;

    [StructLayout(LayoutKind.Sequential)]
    private struct BasicLimits
    {
        public long PerProcessUserTimeLimit, PerJobUserTimeLimit;
        public uint LimitFlags;
        public UIntPtr MinimumWorkingSetSize, MaximumWorkingSetSize;
        public uint ActiveProcessLimit;
        public UIntPtr Affinity;
        public uint PriorityClass, SchedulingClass;
    }
    [StructLayout(LayoutKind.Sequential)]
    private struct IoCounters
    {
        public ulong ReadOperationCount, WriteOperationCount, OtherOperationCount;
        public ulong ReadTransferCount, WriteTransferCount, OtherTransferCount;
    }
    [StructLayout(LayoutKind.Sequential)]
    private struct ExtendedLimits
    {
        public BasicLimits BasicLimitInformation;
        public IoCounters IoInfo;
        public UIntPtr ProcessMemoryLimit, JobMemoryLimit, PeakProcessMemoryUsed, PeakJobMemoryUsed;
    }
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    private static extern SafeFileHandle CreateJobObject(IntPtr attributes, string name);
    [DllImport("kernel32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool SetInformationJobObject(SafeFileHandle handle, int informationClass,
        ref ExtendedLimits information, uint length);
    [DllImport("kernel32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool AssignProcessToJobObject(SafeFileHandle handle, IntPtr process);

    private static void ContainWrapper()
    {
        job = CreateJobObject(IntPtr.Zero, null);
        if (job.IsInvalid) throw new Win32Exception(Marshal.GetLastWin32Error(), "Cannot create owned process job");
        var limits = new ExtendedLimits();
        limits.BasicLimitInformation.LimitFlags = KillOnJobClose;
        if (!SetInformationJobObject(job, ExtendedLimitInformation, ref limits,
            (uint)Marshal.SizeOf(typeof(ExtendedLimits))))
            throw new Win32Exception(Marshal.GetLastWin32Error(), "Cannot set owned process job limits");
        using (var self = Process.GetCurrentProcess())
            if (!AssignProcessToJobObject(job, self.Handle))
                throw new Win32Exception(Marshal.GetLastWin32Error(), "Cannot contain process wrapper");
        // No breakaway flag is enabled. All subsequently created descendants
        // inherit this kernel-owned job even after their parent has exited.
    }

    // Windows CommandLineToArgvW rules, including empty arguments and backslashes
    // before quotes/end. No shell command is evaluated by Process.Start.
    private static string Quote(string value)
    {
        var output = new StringBuilder("\"");
        int slashes = 0;
        foreach (char character in value)
        {
            if (character == '\\') { ++slashes; continue; }
            output.Append('\\', slashes * (character == '"' ? 2 : 1));
            if (character == '"') output.Append('\\');
            output.Append(character);
            slashes = 0;
        }
        output.Append('\\', slashes * 2);
        return output.Append('"').ToString();
    }
    public static void Run(string executable, string[] arguments, string cwd)
    {
        ContainWrapper();
        var quoted = Array.ConvertAll(arguments, Quote);
        var start = new ProcessStartInfo(executable, String.Join(" ", quoted));
        start.UseShellExecute = false;
        start.WorkingDirectory = cwd;
        using (var child = Process.Start(start))
        {
            if (child == null) throw new InvalidOperationException("Owned executable did not start");
            child.WaitForExit();
            int code = child.ExitCode;
            GC.KeepAlive(job);
            Environment.Exit(code);
        }
    }
}
