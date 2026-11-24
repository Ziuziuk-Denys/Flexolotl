# Changelog

## 0.2.0

- Plugins: sources of statuses made by anyone. Install them from the list inside the app, see what each one may do before it runs. Three to start with: Minecraft, game dialogs and Pomodoro. How to write one: the `plugins` folder.

## 0.1.3

- Under 9 MB of memory in the tray with the window closed, it was 44. Closing the window hands the status over to a fresh copy without a blink for your friends.
- Repairs updates: 0.1.2 could not check the signature, so install this version by hand once.

## 0.1.2

- Updates itself: looks for a new version at every start, downloads it quietly and shows "Update" in the header. The status stays on through the restart. Every download is checked against a signature.

## 0.1.1

- A song appears on your profile in under half a second, at once with its cover and the line. It took four seconds.
- Your pictures are kept in a catalogue, one click puts any of them back.
- The cover of the song comes before your own picture.
- A photo is made small before it is uploaded, so it goes out almost at once. Photos from a phone are no longer turned.

## 0.1.0

The first release, a test: tell us what breaks.

- A profile card you edit in place: text, picture or GIF, timer, two buttons.
- One button to show the status to friends and take it off.
- Music and videos from players and browser tabs, with covers and lyrics line by line, also for slowed and sped up versions.
- Your own pictures from the computer, with upload progress.
- Saved statuses with engraved covers and share codes.
- Tray, background run, start with the computer.
- English and Russian.
- Windows, macOS and Linux, x64 and arm64.

Checked by hand on Fedora with Wayland and X11. The Windows and macOS builds start and draw correctly on clean machines, nobody has used them with a live Discord yet.
