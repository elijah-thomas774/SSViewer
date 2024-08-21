# To Use
Make Sure rust is installed to run the main program

The python script requires nlzss11  
`pip install nlzss11`

This viewer required the Stage files to be preprocessed from the game.
- `python pre_process.py -i <STAGE_DIR>`
    - `<STAGE_DIR>` is the stage file directory. An example would be if you have the rando:  
    `python pre_process.py -i "<RANDO_DIR>\actual-extract\DATA\files\Stage"`

Once Stages are preprocessed, just running `cargo run` will run the application :)