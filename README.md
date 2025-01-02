# Ball in a Box
A game about a ball, in a box.

## Credits
- The game uses [Macroquad](https://macroquad.rs/) which was made by [not-fl3](https://github.com/not-fl3).
- The font used is called "Fredericka The Great".
- The balls comes from a toy called "Emoji Splat Ball".
- The rest of the assets and programming was done by me. (Zan)

## Controls
- To move the box, use your mouse to hover over the box, and then hold left/right click and move your cursor. Alternatively, you can click once without moving your cursor, and now you can move your cursor without having to hold down the button.
- To open the menu, click `Esc`, or double left/right click on your mouse without moving it.
- To close the game, open the menu and then click `Quit`. Or you could just do Alt + F4.
- To change ball, simply type in the name of the ball. When you first download the game you will have `grinning` and `distress`.
- To change sounds, simply type in the name of the sound pack. When you first download the game you will have `thud` and `pop`.
- To change assets, simply type in the name of the asset pack. When you first download the game you will have `none` (no asset pack active) and `inverted`.

## Custom balls
To add custom balls, open the `balls` folder and put your image in there, and then type the name of your image inside the game. (No need to restart the game) Just make sure it's a PNG as it will not work with any other format.

## Custom sounds
To add custom sounds, open the `sounds` folder and make a new folder with the name of your sound pack. Then, add your sounds into the folder. (Name doesn't matter) Then type the name of your sound pack inside the game. (No need to restart the game) Just make sure it's an OGG or WAV as it will not work with any other format.

## Custom assets
To add custom assets, open the `asset_packs` folder and make a new folder with the name of your asset pack. Then, add all of your assets into the folder and rename them to the asset you wanna override. (Look inside the `assets` folder) Then type the name of your asset pack inside the game. (No need to restart the game) Just make sure the file names and file formats match.

## Ball is lagging/not synced?
The ball might not smoothly follow the  window. I haven't found a solid way to solve this, but what you could do is to open the menu and click `Settings`. You can then try to set `Max FPS` to the highest and then enable VSync (VSync will automatically cap the frame-rate) and then adjust `Delay frames` to make the window movement synced with the ball. Usually its between 0 and 2.

Or if your monitor is higher than 60fps, you can try only setting `Max FPS` to your monitors frame-rate.

These aren't guaranteed to fix it, so just do whichever one is the least laggy.

## Compiling
When compiling for release I use this command: `RUSTFLAGS="-Zlocation-detail=none -Zfmt-debug=none" cargo +nightly-2024-12-10 build -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --target x86_64-pc-windows-gnu --release`.