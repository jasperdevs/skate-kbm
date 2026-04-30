using System.Collections.Concurrent;
using System.Diagnostics;
using System.Runtime.InteropServices;
using Nefarius.ViGEm.Client;
using Nefarius.ViGEm.Client.Targets;
using Nefarius.ViGEm.Client.Targets.Xbox360;

namespace SkateKbm.Mapper;

internal static class Program
{
    private const int WhKeyboardLl = 13;
    private const int WhMouseLl = 14;
    private const int WmKeyDown = 0x0100;
    private const int WmKeyUp = 0x0101;
    private const int WmSysKeyDown = 0x0104;
    private const int WmSysKeyUp = 0x0105;
    private const int WmMouseMove = 0x0200;
    private const int WmLButtonDown = 0x0201;
    private const int WmLButtonUp = 0x0202;
    private const int WmRButtonDown = 0x0204;
    private const int WmRButtonUp = 0x0205;

    private static readonly ConcurrentDictionary<int, bool> KeyState = new();
    private static readonly object MouseLock = new();
    private static LowLevelKeyboardProc? _keyboardProc;
    private static LowLevelMouseProc? _mouseProc;
    private static IntPtr _keyboardHook;
    private static IntPtr _mouseHook;
    private static int _lastMouseX;
    private static int _lastMouseY;
    private static int _mouseDx;
    private static int _mouseDy;
    private static bool _leftMouse;
    private static bool _rightMouse;
    private static bool _hasMousePosition;

    public static int Main(string[] args)
    {
        if (args.Contains("--help") || args.Contains("-h"))
        {
            PrintHelp();
            return 0;
        }

        var config = MapperConfig.FromArgs(args);

        using var client = new ViGEmClient();
        IXbox360Controller controller;
        try
        {
            controller = client.CreateXbox360Controller();
            controller.Connect();
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine("error: could not connect virtual Xbox 360 controller");
            Console.Error.WriteLine("error: install ViGEmBus, then run skate-kbm again");
            Console.Error.WriteLine($"detail: {ex.Message}");
            return 2;
        }

        Console.CancelKeyPress += (_, e) =>
        {
            e.Cancel = true;
            Running = false;
        };

        _keyboardProc = KeyboardHook;
        _mouseProc = MouseHook;
        _keyboardHook = SetHook(WhKeyboardLl, _keyboardProc);
        _mouseHook = SetHook(WhMouseLl, _mouseProc);

        if (_keyboardHook == IntPtr.Zero || _mouseHook == IntPtr.Zero)
        {
            Console.Error.WriteLine("error: failed to install keyboard/mouse hooks");
            return 1;
        }

        Console.WriteLine("status: connected virtual Xbox 360 controller");
        Console.WriteLine("status: press Ctrl+C to stop");

        var stopwatch = Stopwatch.StartNew();
        var lastStatus = TimeSpan.Zero;

        while (Running)
        {
            Application.DoEvents();
            Submit(controller, config);
            Thread.Sleep(8);

            if (stopwatch.Elapsed - lastStatus > TimeSpan.FromSeconds(1))
            {
                lastStatus = stopwatch.Elapsed;
                Console.WriteLine($"state: wasd={Pressed(Vk.W)}{Pressed(Vk.A)}{Pressed(Vk.S)}{Pressed(Vk.D)} mouse={_mouseDx},{_mouseDy} buttons={(_leftMouse ? "L" : "-")}{(_rightMouse ? "R" : "-")}");
            }
        }

        controller.ResetReport();
        controller.SubmitReport();
        controller.Disconnect();
        UnhookWindowsHookEx(_keyboardHook);
        UnhookWindowsHookEx(_mouseHook);
        Console.WriteLine("status: stopped");
        return 0;
    }

    private static bool Running { get; set; } = true;

    private static void Submit(IXbox360Controller controller, MapperConfig config)
    {
        short lx = 0;
        short ly = 0;

        if (IsDown(Vk.A)) lx -= short.MaxValue;
        if (IsDown(Vk.D)) lx += short.MaxValue;
        if (IsDown(Vk.W)) ly += short.MaxValue;
        if (IsDown(Vk.S)) ly -= short.MaxValue;

        int dx;
        int dy;
        bool left;
        bool right;
        lock (MouseLock)
        {
            dx = _mouseDx;
            dy = _mouseDy;
            _mouseDx = 0;
            _mouseDy = 0;
            left = _leftMouse;
            right = _rightMouse;
        }

        short rx = ClampStick(dx * config.MouseSensitivity);
        short ry = ClampStick(-dy * config.MouseSensitivity);

        controller.SetAxisValue(Xbox360Axis.LeftThumbX, lx);
        controller.SetAxisValue(Xbox360Axis.LeftThumbY, ly);
        controller.SetAxisValue(Xbox360Axis.RightThumbX, rx);
        controller.SetAxisValue(Xbox360Axis.RightThumbY, ry);

        SetButton(controller, Xbox360Button.A, IsDown(Vk.LeftShift) || IsDown(Vk.RightShift) || IsDown(Vk.Space));
        SetButton(controller, Xbox360Button.B, IsDown(Vk.Escape));
        SetButton(controller, Xbox360Button.X, IsDown(Vk.E));
        SetButton(controller, Xbox360Button.Y, IsDown(Vk.R));
        SetButton(controller, Xbox360Button.LeftShoulder, IsDown(Vk.Q));
        SetButton(controller, Xbox360Button.RightShoulder, IsDown(Vk.F));
        SetButton(controller, Xbox360Button.Back, IsDown(Vk.Tab));
        SetButton(controller, Xbox360Button.Start, IsDown(Vk.Enter));
        SetButton(controller, Xbox360Button.LeftThumb, IsDown(Vk.LeftControl) || IsDown(Vk.RightControl));
        SetButton(controller, Xbox360Button.RightThumb, IsDown(Vk.C));
        SetButton(controller, Xbox360Button.Up, IsDown(Vk.Up));
        SetButton(controller, Xbox360Button.Down, IsDown(Vk.Down));
        SetButton(controller, Xbox360Button.Left, IsDown(Vk.Left));
        SetButton(controller, Xbox360Button.Right, IsDown(Vk.Right));

        controller.SetSliderValue(Xbox360Slider.LeftTrigger, right ? byte.MaxValue : (byte)0);
        controller.SetSliderValue(Xbox360Slider.RightTrigger, left ? byte.MaxValue : (byte)0);
        controller.SubmitReport();
    }

    private static void SetButton(IXbox360Controller controller, Xbox360Button button, bool pressed)
    {
        controller.SetButtonState(button, pressed);
    }

    private static short ClampStick(int value)
    {
        if (value > short.MaxValue) return short.MaxValue;
        if (value < short.MinValue) return short.MinValue;
        return (short)value;
    }

    private static bool IsDown(int key) => KeyState.TryGetValue(key, out var down) && down;
    private static string Pressed(int key) => IsDown(key) ? "1" : "0";

    private static IntPtr SetHook(int hookType, Delegate proc)
    {
        using var currentProcess = Process.GetCurrentProcess();
        using var currentModule = currentProcess.MainModule;
        return SetWindowsHookEx(hookType, proc, GetModuleHandle(currentModule?.ModuleName), 0);
    }

    private static IntPtr KeyboardHook(int nCode, IntPtr wParam, IntPtr lParam)
    {
        if (nCode >= 0)
        {
            int vkCode = Marshal.ReadInt32(lParam);
            int message = wParam.ToInt32();
            if (message == WmKeyDown || message == WmSysKeyDown)
                KeyState[vkCode] = true;
            else if (message == WmKeyUp || message == WmSysKeyUp)
                KeyState[vkCode] = false;
        }
        return CallNextHookEx(_keyboardHook, nCode, wParam, lParam);
    }

    private static IntPtr MouseHook(int nCode, IntPtr wParam, IntPtr lParam)
    {
        if (nCode >= 0)
        {
            var data = Marshal.PtrToStructure<Msllhookstruct>(lParam);
            int message = wParam.ToInt32();
            lock (MouseLock)
            {
                if (message == WmMouseMove)
                {
                    if (_hasMousePosition)
                    {
                        _mouseDx += data.pt.x - _lastMouseX;
                        _mouseDy += data.pt.y - _lastMouseY;
                    }
                    _lastMouseX = data.pt.x;
                    _lastMouseY = data.pt.y;
                    _hasMousePosition = true;
                }
                else if (message == WmLButtonDown) _leftMouse = true;
                else if (message == WmLButtonUp) _leftMouse = false;
                else if (message == WmRButtonDown) _rightMouse = true;
                else if (message == WmRButtonUp) _rightMouse = false;
            }
        }
        return CallNextHookEx(_mouseHook, nCode, wParam, lParam);
    }

    private static void PrintHelp()
    {
        Console.WriteLine("""
        skate-kbm-mapper

        Creates a virtual Xbox 360 controller and maps keyboard/mouse input.

        Options:
          --mouse-sensitivity <number>   Right-stick mouse sensitivity. Default: 220
          -h, --help                     Show help
        """);
    }

    private delegate IntPtr LowLevelKeyboardProc(int nCode, IntPtr wParam, IntPtr lParam);
    private delegate IntPtr LowLevelMouseProc(int nCode, IntPtr wParam, IntPtr lParam);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern IntPtr SetWindowsHookEx(int idHook, Delegate lpfn, IntPtr hMod, uint dwThreadId);

    [DllImport("user32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool UnhookWindowsHookEx(IntPtr hhk);

    [DllImport("user32.dll")]
    private static extern IntPtr CallNextHookEx(IntPtr hhk, int nCode, IntPtr wParam, IntPtr lParam);

    [DllImport("kernel32.dll", CharSet = CharSet.Auto, SetLastError = true)]
    private static extern IntPtr GetModuleHandle(string? lpModuleName);

    [StructLayout(LayoutKind.Sequential)]
    private struct Point
    {
        public int x;
        public int y;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct Msllhookstruct
    {
        public Point pt;
        public uint mouseData;
        public uint flags;
        public uint time;
        public IntPtr dwExtraInfo;
    }
}

internal sealed record MapperConfig(int MouseSensitivity)
{
    public static MapperConfig FromArgs(string[] args)
    {
        int sensitivity = 220;
        for (int i = 0; i < args.Length; i++)
        {
            if (args[i] == "--mouse-sensitivity" && i + 1 < args.Length && int.TryParse(args[i + 1], out var value))
            {
                sensitivity = Math.Clamp(value, 1, 2000);
                i++;
            }
        }
        return new MapperConfig(sensitivity);
    }
}

internal static class Vk
{
    public const int Tab = 0x09;
    public const int Enter = 0x0D;
    public const int Escape = 0x1B;
    public const int Space = 0x20;
    public const int Left = 0x25;
    public const int Up = 0x26;
    public const int Right = 0x27;
    public const int Down = 0x28;
    public const int A = 0x41;
    public const int C = 0x43;
    public const int D = 0x44;
    public const int E = 0x45;
    public const int F = 0x46;
    public const int Q = 0x51;
    public const int R = 0x52;
    public const int S = 0x53;
    public const int W = 0x57;
    public const int LeftShift = 0xA0;
    public const int RightShift = 0xA1;
    public const int LeftControl = 0xA2;
    public const int RightControl = 0xA3;
}
