Tool: "DeskFrog"

Core idea:
- a small frog emoji 🐸 (unicode U+1F438), that slowly wanders over the my screen
- it sometimes comments some random short messages like "Why is your mouse so fast?" or "I have important frog business." in a small speech bubble (with a monospace typeface "DOS"-like aesthetic).
- it sometimes sleeps for a short moment (speech bubble says "z z z" or "zZzZz")
- occasionally it would startle
- movement is in steps, *as if* the screen would be tiled (like a character in a traditional roguelike game would move, like e.g. Rogue, TOME, or Dwarf Fortress)
- when my mouse cursor comes close it will run away to another part of the screen and 
- multi screen support, support for huge resolutions
- size of the frog should also roughly be the size of a character in a traditional roguelike. Speech bubble and should also be quite small.
- no graphical assets! no sound!
- the frog should be "on to of everything" (but not in the way of anything)


Tech choices:
- when "frog" runs, a tray icon should appear in the taskbar, which allows me to "quit" frog.
- ideally running on Linux, too, but this is a "bonus" feature. (Mac support would be cool, too, but i do *not* have a Mac to verify anything)
- the frog app should be extremely lightweight
- Tech stack:
	-> I am not sure about the stack, but the two ideas i have are "Rust + egui" / "Rust + winit + wgpu" or "C# + WinUI 3 / Windows App SDK". Rust appears considerably more lightweight to me than C#.
	-> Architecture could be a three part:
		- (1) Frog Brain: State machine, Movement, Commentary
		- (2) Frog Presentation
		- (3) OS Adapter (For Windows: HWND position, transparency, monitor bounds, mouse events, ..)
	-> I am open for suggestions on the stack! No strong preference.
	Note: I currently have no C# pipeline installed, but Rust is already available.