from glob import glob

def main(args):
    from pathlib import Path
    import struct
    mask = int(args.mask, base=16)
    shift = int(args.shift, base=10)
    value = int(args.value, base=16)
    code = int(args.code, base=10)
    input = Path(args.input)

    def check_plc(data : bytearray):
        assert(struct.unpack_from(">4s", data, 0)[0] == b'SPLC')
        assert(struct.unpack_from(">H", data, 4)[0] == 0x14)
        assert(code >= 0 and code <= 4)
        for i in range(0, struct.unpack_from(">H", data, 6)[0]):
            offset =  8 + 0x14 * i

            codes = list(struct.unpack_from(">5I", data, offset))

            if (codes[code] >> shift) & mask == value:
                return True
        
        return False

    # Get All PLCS and check for codes
    for plc in glob("**/*.plc", root_dir=input, recursive=True):
        with open(input / plc, "rb") as f:
            plc_dat = f.read()
            if check_plc(plc_dat):
                print(plc)

if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser()
    parser.add_argument('-i', '--input', required=True)
    parser.add_argument('-c', '--code', required=True)
    parser.add_argument('-m', '--mask', required=True)
    parser.add_argument('-s', '--shift', required=True)
    parser.add_argument('-v', '--value', required=True)


    import sys
    args = parser.parse_args(sys.argv[1:])


    main(args)
    

    
