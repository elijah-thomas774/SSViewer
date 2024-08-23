



from io import BytesIO

DEBUG = False

def main(args):
    from pathlib import Path
    import struct
    mask = int(args.mask, base=16)
    shift = int(args.shift, base=10)
    value = int(args.value, base=16)
    code = int(args.code, base=10)
    index = int(args.index, base=10)
    input = Path(args.input)
    output = Path(args.output)

    file_data = None
    with open(input, "rb") as f:
        file_data = f.read()
    
    if not file_data:
        print(f"Could not read file: {input}")
        return

    if input.name.endswith(".LZ"):
        import nlzss11
        file_data = nlzss11.decompress(file_data)

    def change_plc(data : bytearray):
        assert(struct.unpack_from(">4s", data, 0)[0] == b'SPLC')
        assert(struct.unpack_from(">H", data, 4)[0] == 0x14)
        assert(index == -1 or (struct.unpack_from(">H", (data, 6)[0]) > index and 0 <= index))
        assert(code >= 0 and code <= 4)
        if index != -1:

            offset =  8 + 0x14 * index

            codes = list(struct.unpack_from(">5I", data, offset))
            clear_mask = ~(mask << shift)
            codes[code] = (codes[code] & clear_mask) | ((mask & value) << shift)
            code_bytes = struct.pack(">5I", codes)
            data[offset : offset+0x14] = code_bytes
        else:
            for i in range(0, struct.unpack_from(">H", data, 6)[0]):
                offset =  8 + 0x14 * i

                codes = list(struct.unpack_from(">5I", data, offset))

                if DEBUG:
                    if (codes[code] >> shift) & mask == value:
                        print(f"[{i}, 0x{offset:04X}]", end=" ")
                

                clear_mask = (mask << shift)
                codes[code] = (codes[code] & ~clear_mask) | ((mask & value) << shift)
                code_bytes = struct.pack(">5I", codes[0], codes[1], codes[2], codes[3], codes[4])
                data[offset : offset+0x14] = code_bytes

    from sslib import U8File

    # Load archive
    archive = U8File.parse_u8(BytesIO(file_data))
    
    # Goto Rarc
    for path in archive.get_all_paths():
        if path.split("/")[-1].endswith(".plc"):
            if DEBUG:
                print(path, end="")
            plc_dat = bytearray(archive.get_file_data(path))
            change_plc(plc_dat)
            archive.set_file_data(path, plc_dat)
            if DEBUG:
                print("\n", end="")
        if path.split("/")[1] == "rarc":
            room_dat = U8File.parse_u8(BytesIO(archive.get_file_data(path)))
            for r_path in room_dat.get_all_paths():
                if r_path.split("/")[-1].endswith(".plc"):
                    if DEBUG:
                        print(path + r_path, end="")
                    plc_dat = bytearray(room_dat.get_file_data(r_path))
                    change_plc(plc_dat)
                    room_dat.set_file_data(r_path, plc_dat)
                    if DEBUG:
                        print("\n", end="")
            archive.set_file_data(path, bytes(room_dat.to_buffer()))
        # Only Check objects on debug
        if DEBUG:
            if path.split("/")[1] == "oarc":
                obj_dat = U8File.parse_u8(BytesIO(archive.get_file_data(path)))
                for o_path in obj_dat.get_all_paths():
                    if o_path.split("/")[-1].endswith(".plc"):
                        if DEBUG:
                            print(path + o_path, end="")
                        plc_dat = bytearray(obj_dat.get_file_data(o_path))
                        change_plc(plc_dat)
                        obj_dat.set_file_data(o_path, plc_dat)
                        if DEBUG:
                            print("\n", end="")
                archive.set_file_data(path, bytes(obj_dat.to_buffer()))
    
    file_data = bytearray(archive.to_buffer())

    if output.name.endswith(".LZ"):
        import nlzss11
        file_data = nlzss11.compress(file_data)

    if not DEBUG:
        with open(output, "wb") as f:
            f.write(file_data)

if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser()
    parser.add_argument('-i', '--input', required=True)
    parser.add_argument('-o', '--output', required=True)
    parser.add_argument('-c', '--code', required=True)
    parser.add_argument('-m', '--mask', required=True)
    parser.add_argument('-s', '--shift', required=True)
    parser.add_argument('-v', '--value', required=True)
    parser.add_argument('--index', default="-1")


    import sys
    args = parser.parse_args(sys.argv[1:])

    main(args)
    

    
