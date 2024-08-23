from io import BytesIO
from pathlib import Path
from sslib import U8File
import struct;
import os
import nlzss11

class node(object):
    def __init__(self, value, children = []):
        self.value = value
        self.children = children

    def __str__(self, level=0):
        ret = "\t"*level+repr(hex(self.value))+"\n"
        for i, child in enumerate(self.children):
            ret +=  child.__str__(level+1)
        return ret

    def __repr__(self):
        return '<tree node representation>'


def read_node(value, buffer):
    
    children : list[node] = []
    for i in range(8):
        new_val = struct.unpack_from(">i", buffer, value + i * 4)[0]
        if new_val < 0:
            children.append(node(value + new_val))
        else:
            children.append(read_node(value + new_val, buffer))
        
    return node(value, children)

def read_octree():
    file = "room.kcl"
    with open(file, "br") as f:
        file_data = f.read()
        octree_offset = struct.unpack_from(">I", file_data, 0xC)[0]
        octree = read_node(octree_offset, file_data)
        print(octree)


def preprocess_stages(data_dir):
    output_dir = Path("Collision Files")

    # If output_dir doesnt exist
    if not os.path.exists(output_dir):
        os.makedirs(output_dir)

    stage_dir = data_dir / "Stage"

    # List of Stages
    stages = os.listdir(stage_dir)
    stages_l0 : list[Path] = []

    # Validate finding stages
    for stage in stages:
        stage_path = stage_dir / stage

        layer_0 = stage_path / f"{stage}_stg_l0.arc.LZ"
        if not os.path.exists(layer_0):
            layer_0 = stage_path / f"{stage}_stg_l0.arc"
            if not os.path.exists(layer_0):
                layer_0 = stage_path / "NX" / f"{stage}_stg_l0.arc.LZ"
                if not os.path.exists(layer_0):
                    layer_0 = stage_path / "NX" / f"{stage}_stg_l0.arc"
                    if not os.path.exists(layer_0):
                        return
        
        stages_l0.append(layer_0)

    # For Each stage
    #   Decompress the File
    #   Read the Archive
    #   Grab need files
    #   Create a new dir holding the files
    for (stage, layer_0) in zip(stages, stages_l0):
        # Open file and get data
        with open(layer_0, "rb") as f:
            data = f.read()
            if layer_0.name.endswith(".LZ"):
                print(f"Decompressing {layer_0}...")
                data = nlzss11.decompress(data)
        if len(data) == 0:
            print(f"ERR: Unable to Process {stage}: Could not read or decompress data")
            continue

        l_arc = U8File.parse_u8(BytesIO(data))
        layer_paths = l_arc.get_all_paths()
        

        plc_dat = []
        dzb_dat = []
        
        rarc_dat = []
        oarc_dat = []

        for l_path in layer_paths:
            split_path = l_path.split("/")
            l_folder_type = split_path[1]
            l_file_name = split_path[-1]
            if l_folder_type == "dat":
                if l_file_name.endswith(".plc"):
                    plc_dat.append((l_file_name, l_arc.get_file_data(l_path)))
            elif l_folder_type == "dzb":
                dzb_dat.append((l_file_name, l_arc.get_file_data(l_path)))
            elif l_folder_type == "rarc":
                r_arc = U8File.parse_u8(BytesIO(l_arc.get_file_data(l_path)))
                r_paths = r_arc.get_all_paths()

                r_dat = []
                r_kcl = []
                
                for r_path in r_paths:
                    split_path = r_path.split("/")
                    r_folder_type = split_path[1]
                    r_file_name = split_path[-1]
                    if r_folder_type == "dat":
                        if r_file_name.endswith(".plc"):
                            r_dat.append((r_file_name, r_arc.get_file_data(r_path)))
                    elif r_folder_type == "kcl":
                        r_kcl.append((r_file_name, r_arc.get_file_data(r_path)))
                
                # split path again because file_name was overriden
                room_id = int(l_file_name[-6:-4]) # r##
                room_name = f"Room {room_id}"
                rarc_dat.append((room_name, r_dat, r_kcl))
            
            elif l_folder_type == "oarc":
                o_arc = U8File.parse_u8(BytesIO(l_arc.get_file_data(l_path)))
                o_paths = o_arc.get_all_paths()

                o_dat = []

                for o_path in o_paths:
                    split_path = o_path.split("/")
                    o_folder_type = split_path[1]
                    o_file_name = split_path[-1]
                    if o_file_name.endswith(".plc"):
                        o_dat.append((o_file_name, o_arc.get_file_data(o_path)))
                    elif o_file_name.endswith(".dzb"):
                        o_dat.append((o_file_name, o_arc.get_file_data(o_path)))
                if len(o_dat) != 0:
                    oarc_dat.append((l_file_name[:-4], o_dat))

        object_dir = data_dir / "Object"

        if os.path.exists( object_dir / "NX"):
            object_dir = object_dir / "NX"

        object_pack = object_dir / "ObjectPack.arc.LZ"

        with open(object_pack, "rb") as f:
            data = f.read()
        
        data = nlzss11.decompress(data)

        archive = U8File.parse_u8(BytesIO(data))
        paths = archive.get_all_paths()
        for path in paths:
            split_path = path.split("/")
            folder_type = split_path[1]
            file_name = split_path[-1]

            if folder_type == "oarc":
                o_arc = U8File.parse_u8(BytesIO(archive.get_file_data(path)))
                o_paths = o_arc.get_all_paths()

                o_dat = []

                for o_path in o_paths:
                    split_path = o_path.split("/")
                    o_folder_type = split_path[1]
                    o_file_name = split_path[-1]
                    if o_file_name.endswith(".plc"):
                        o_dat.append((o_file_name, o_arc.get_file_data(o_path)))
                    elif o_file_name.endswith(".dzb"):
                        o_dat.append((o_file_name, o_arc.get_file_data(o_path)))
                if len(o_dat) != 0:
                    oarc_dat.append((file_name[:-4], o_dat))

        
        # new layout will be
        #   <stage>
        #     stage
        #       dzb/plc pairing
        #     rooms
        #       r##
        #        klc/plc pairing
        #   Oarc
        #     Objname
        #       dzb/plc pairing
        curr_dir = output_dir / stage / "addon"
        os.makedirs(curr_dir, exist_ok=True)
        for stage_files in (dzb_dat + plc_dat):
            with open(curr_dir / stage_files[0], "wb") as f:
                f.write(stage_files[1])          

        curr_dir = output_dir / stage / "rooms"
        os.makedirs(curr_dir, exist_ok=True)
        for rooms in rarc_dat:
            os.makedirs(curr_dir / rooms[0], exist_ok=True)
            for room_files in (rooms[1] + rooms[2]):
                with open(curr_dir / rooms[0] / room_files[0], "wb") as f:
                    if (room_files[1]):
                        f.write(room_files[1])
                    else:
                        print(room_files[0] + " has no data")
        
        curr_dir = output_dir / "Oarc"
        os.makedirs(curr_dir, exist_ok=True)
        for obj in oarc_dat:
            os.makedirs(curr_dir / obj[0], exist_ok=True)
            for obj_files in obj[1]:
                with open(curr_dir / obj[0] / obj_files[0], "wb") as f:
                    f.write(obj_files[1])       

if __name__ == '__main__':
    import sys
    import argparse
    parser  = argparse.ArgumentParser()
    parser.add_argument('-i', '--input', help="Input Source dir for Skyward Sword Files. Must point to the `<SSHD_EXTRACT>\\romfs` or `<SS_EXTRACT>\\DATA\\files` directory", required=True)

    args = parser.parse_args(sys.argv[1:])
    print(args)

    preprocess_stages(Path(args.input))
    