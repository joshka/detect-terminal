# Terminal environment fixtures

These fixtures model child process environments assembled by terminal source code. UUIDs, process
IDs, pane IDs, and versions in the tests are representative values in the formats used upstream.
They are not captures from a running Windows or Linux machine. Source links are pinned to commits.

| Terminal         | Source                   | Main fixture marker      |
| ---------------- | ------------------------ | ------------------------ |
| Windows Terminal | [connection][wt]         | `WT_SESSION`             |
| WezTerm          | [config][wez-config]     | `TERM_PROGRAM=WezTerm`   |
| Alacritty        | [PTY][alacritty-pty]     | `TERM=alacritty`         |
| Kitty            | [child][kitty-child]     | `KITTY_WINDOW_ID`        |
| Tilix            | [environment][tilix-env] | `TILIX_ID`               |
| GNOME Terminal   | [screen][gnome-screen]   | `GNOME_TERMINAL_SERVICE` |
| Konsole          | [session][konsole]       | `KONSOLE_VERSION`        |
| foot             | [slave][foot-slave]      | `TERM=foot`              |
| Terminator       | [terminal][terminator]   | `TERMINATOR_UUID`        |
| ConEmu           | [environment][conemu]    | `ConEmuPID`              |
| Cmder            | [launcher][cmder-launch] | `CMDER_ROOT`             |
| mintty           | [child][mintty-child]    | `TERM_PROGRAM=mintty`    |

## Apple Terminal color

Terminal.app's `TERM_PROGRAM_VERSION` is its build. These are third-party reports, not captures made
by this project. A reported `COLORTERM` could have been exported by the user, though build 466's
[capture][apple-466] shows it among Terminal's other variables.

| Build | macOS        | RGB rendering                                | `COLORTERM`                  |
| ----- | ------------ | -------------------------------------------- | ---------------------------- |
| 455.1 | 15.8         | [Misrenders][apple-455], sampled             | Unset                        |
| 464   | 26.0 (Tahoe) | [Renders][apple-464], checked by eye         | [`truecolor`][apple-464-env] |
| 466   | 26.2–26.3    | Disputed: [no][apple-zsh], [yes][apple-raku] | [`truecolor`][apple-466]     |
| 470.2 | 26           | [Renders][apple-470], gradient test          | `truecolor` reported         |

Build 466's negative report gives no method. No public release was found using builds 456 through
463. The detector treats every build before 464 as 256-color, so at worst an early Tahoe beta uses
indexed colors. Build 464 is the first reported to work; 470 is the first uncontested.

[apple-455]: https://github.com/vshylov/mindfork-rs/pull/716
[apple-464]: https://github.com/ratatui/ratatui/issues/1972
[apple-464-env]: https://github.com/iwadon/terse/blob/main/docs/research/terminal-inspection.md
[apple-466]: https://github.com/kishankalandoor/DevOps/blob/main/env.html
[apple-470]: https://github.com/OmarMusayev/terminaltui/blob/main/src/style/colors.ts
[apple-raku]: https://github.com/japhb/Terminal-Capabilities/commit/0393e87db4d0c33f211c65fa3cc04467741ebd33
[apple-zsh]: https://www.zsh.org/mla/workers/2026/msg00252.html

## Other color evidence

- tmux 3.2 first sets `TERM_PROGRAM=tmux` and its version for panes; 3.6 also
  [sets `COLORTERM=truecolor`][tmux-env]. Local checks of 3.2, 3.3a, 3.4, 3.5a, and 3.8 with
  simulated outer environments passed RGB through for Ghostty and converted it to indexed color for
  a plain `xterm-256color` client.
- The termstandard list says Windows Terminal supports [24-bit color][wt-color].
- The same list says iTerm2 has supported RGB [since version 3][iterm-color].

[iterm-color]: https://github.com/termstandard/colors
[tmux-env]: https://github.com/tmux/tmux/blob/3.6/environ.c
[wt-color]: https://github.com/termstandard/colors

## Windows Terminal

Its [connection code][wt] adds `WT_SESSION` as a GUID and `WT_PROFILE_ID` as a braced GUID, then
passes both through `WSLENV`. Detection selects Windows Terminal from a nonempty `WT_SESSION`.

When Windows Terminal [attaches as the default console host][wt-default] after the shell starts, it
cannot inject `WT_SESSION` into that already-running process. Detection then returns `Unknown` or a
`TERM` emulation family if no other marker matches. `TERM_PROGRAM=WindowsTerminal` is a
[user-configured profile value][wt-profile], not a variable injected by this connection path.

## WezTerm

Its [config][wez-config] sets `TERM_PROGRAM=WezTerm`, a version, `COLORTERM=truecolor`, and
configured `TERM`; [pane creation][wez-pane] adds `WEZTERM_PANE`. Windows and WSL receive `WSLENV`
entries for the terminal variables. The fixture uses the default `TERM=xterm-256color`, but the
program marker makes detection select WezTerm rather than Xterm.

## Alacritty

Its [PTY setup][alacritty-pty] sets `COLORTERM=truecolor` and `TERM=alacritty` only when that
terminfo entry exists. Otherwise it uses `TERM=xterm-256color`. Its [Unix IPC code][alacritty-ipc]
can add `ALACRITTY_SOCKET`. With no socket or program marker, that fallback makes detection return
Xterm; Alacritty is not identifiable from the shared terminfo name.

## Kitty

Its [child setup][kitty-child] adds `KITTY_PID`, configured `TERM`, and `COLORTERM=truecolor`;
[window creation][kitty-window] adds `KITTY_WINDOW_ID`. It removes inherited `VTE_VERSION` before
launching children. The fixture selects Kitty from `KITTY_WINDOW_ID`.

## Tilix

Its [environment assembly][tilix-env] adds `TILIX_ID`, `TERM=xterm-256color`, and numeric
`VTE_VERSION`, then [copies many parent variables][tilix-inherit]. If `WT_SESSION` survives into a
Tilix session without a recognized `TERM_PROGRAM`, detection selects Windows Terminal before
reaching `TILIX_ID`. This is a possible inherited-variable outcome, not a live observation.

## GNOME Terminal

Its [screen setup][gnome-screen] adds `GNOME_TERMINAL_SERVICE` and `GNOME_TERMINAL_SCREEN` after
removing inherited variables including `WT_SESSION`, `TMUX`, `STY`, `TERM`, and `VTE_VERSION`; see
its [filters][gnome-filters]. The fixture selects GNOME Terminal from the service marker.

## Konsole

Its [session setup][konsole] adds numeric `KONSOLE_VERSION` to the profile environment. The
fixture's six-digit value follows the source's version format and selects Konsole before the
fixture's `TERM=xterm-256color` value.

## foot

Its [slave setup][foot-slave] sets configured `TERM` and `COLORTERM=truecolor`. The
[build default][foot-default] changes from `foot` to `xterm-256color` without foot terminfo. With
only that shared `TERM`, detection returns Xterm rather than foot.

## Terminator

Its [terminal setup][terminator] adds configured `TERM`, `COLORTERM`, and `TERMINATOR_UUID`. The
fixture selects Terminator from its UUID rather than the shared `TERM` value.

## ConEmu

Its [environment setup][conemu] sets `ConEmuHWND` as a hexadecimal handle and `ConEmuPID` as a
decimal process ID. The fixture selects ConEmu from the process ID. If `CMDER_ROOT` remains set from
an earlier Cmder session, detection selects Cmder first. This is a possible inherited-variable
outcome, not a live observation.

## Cmder

Its [launcher][cmder-launch] sets `CMDER_ROOT` before starting ConEmu; its [init script][cmder-init]
can also set the root. A Cmder session can carry both Cmder and ConEmu markers. The fixture selects
Cmder because `CMDER_ROOT` precedes `ConEmuPID`.

## mintty

Its [child setup][mintty-child] sets configured `TERM`, `TERM_PROGRAM=mintty`, and
`TERM_PROGRAM_VERSION`. The fixture selects mintty from the program marker. The source cautions that
environment identity can be unreliable.

## GNU Screen

The detector recognizes `TERM=screen*` as Screen emulation. That terminfo name alone does not prove
that a Screen session is active. With no multiplexer marker, it still yields a Screen result.

## tmux

tmux can use `TERM=screen*`. A nonempty `TMUX` marker prevents the fallback from claiming Screen. An
opted-in client probe can recover a client terminfo name, but several attached clients can make the
selected client ambiguous.

## Zellij

Zellij can also use the `screen` terminfo family. A nonempty `ZELLIJ` marker prevents the fallback
from claiming Screen; if that marker is absent, the same `TERM` alone can yield Screen.

These fixtures establish how detection responds to source-backed inputs. They do not establish how
every terminal version or launch route configures its environment.

[wt]: https://github.com/microsoft/terminal/blob/bb0541e7a972a1f9e39318c1f6b4e70347a86696/src/cascadia/TerminalConnection/ConptyConnection.cpp#L55-L98
[wt-default]: https://github.com/microsoft/terminal/wiki/Frequently-Asked-Questions-%28FAQ%29
[wez-config]: https://github.com/wezterm/wezterm/blob/b09b56c29c1e367e598b60ca266e2cc9038751e0/config/src/config.rs#L1600-L1623
[wez-pane]: https://github.com/wezterm/wezterm/blob/b09b56c29c1e367e598b60ca266e2cc9038751e0/mux/src/domain.rs#L475-L485
[alacritty-pty]: https://github.com/alacritty/alacritty/blob/d692748d3f61253ebe9f5094320120d22f6a046f/alacritty_terminal/src/tty/mod.rs#L98-L109
[alacritty-ipc]: https://github.com/alacritty/alacritty/blob/d692748d3f61253ebe9f5094320120d22f6a046f/alacritty/src/polling/ipc.rs#L28-L45
[kitty-child]: https://github.com/kovidgoyal/kitty/blob/4136c93c38d7125e10ae8560545c31ce93f6bc36/kitty/child.py#L375-L390
[kitty-window]: https://github.com/kovidgoyal/kitty/blob/4136c93c38d7125e10ae8560545c31ce93f6bc36/kitty/tabs.py#L773-L780
[tilix-env]: https://github.com/gnunn1/tilix/blob/46b6a7c6b318257bba145d527cbf4a09fa9fbb43/source/gx/tilix/terminal/terminal.d#L2734-L2740
[tilix-inherit]: https://github.com/gnunn1/tilix/blob/46b6a7c6b318257bba145d527cbf4a09fa9fbb43/source/gx/tilix/terminal/terminal.d#L2858-L2885
[gnome-screen]: https://github.com/GNOME/gnome-terminal/blob/b23813aa7d9fbfa978f8389a8ba0f738421e76c1/src/terminal-screen.cc#L1935-L1965
[gnome-filters]: https://github.com/GNOME/gnome-terminal/blob/b23813aa7d9fbfa978f8389a8ba0f738421e76c1/src/terminal-client-utils.cc#L230-L258
[konsole]: https://github.com/KDE/konsole/blob/0a86b379f87cca2a2432dbc0a9a4ea73159f20ab/src/session/SessionManager.cpp#L184-L208
[foot-slave]: https://github.com/r-c-f/foot/blob/1dabc10494ac6626398c99507b791a21b923a616/slave.c#L287-L291
[foot-default]: https://github.com/r-c-f/foot/blob/1dabc10494ac6626398c99507b791a21b923a616/config.h#L10-L16
[terminator]: https://github.com/gnome-terminator/terminator/blob/9f2d0b6ccc2c0f9be10ba18f85df90acb413fc39/terminatorlib/terminal.py#L1718-L1730
[conemu]: https://github.com/ConEmu/ConEmu/blob/c8cebb921e69d26fde43ba0a2af79115d1a07977/src/common/SetEnvVar.cpp#L39-L67
[cmder-launch]: https://github.com/cmderdev/cmder/blob/1d6777b5c087c7e31ab9b2a1237276c9fe8a0d3a/launcher/src/CmderLauncher.cpp
[cmder-init]: https://github.com/cmderdev/cmder/blob/1d6777b5c087c7e31ab9b2a1237276c9fe8a0d3a/vendor/init.bat#L32-L45
[mintty-child]: https://github.com/mintty/mintty/blob/6696ab14fdb9ce0082b3a6ccbf0be21fdbbf3f5a/src/child.c#L572-L579
[wt-profile]: https://github.com/microsoft/terminal/issues/20242
