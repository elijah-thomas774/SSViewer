# To Use
Make Sure rust is installed to run the main program

The python script requires nlzss11  
`pip install nlzss11`

This viewer required the Stage files to be preprocessed from the game.
- `python pre_process.py -i <SS_EXTRACT_FILES>`
    - `<SS_EXTRACT_FILES>` is the files or romfs directory. Examples would be:
        - SD:`python pre_process.py -i "<RANDO_DIR>\actual-extract\DATA\files"`
        - HD:`python pre_process.py -i "<HD_EXTRACT>\romfs"`

Once Stages are preprocessed, just running `cargo run` will run the application. (Ensure rust is updated via `rustup update`) :)

# Controls

- `WASD` to move around
- `Shift` to descend
- `Space` to ascend
- `-` to shrink stage
- `+` to expand stage
- `Click and Drag` to pan camera
- Slider is currently move speed