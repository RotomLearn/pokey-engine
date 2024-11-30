import json
import math
from collections import defaultdict

from pokey_engine import Move, Pokemon, Side, SideConditions, State


def normalize_name(name):
    return (
        name.replace(" ", "")
        .replace("-", "")
        .replace(".", "")
        .replace("'", "")
        .replace("%", "")
        .replace("*", "")
        .replace(":", "")
        .strip()
        .lower()
        .encode("ascii", "ignore")
        .decode("utf-8")
    )


class Utilities:
    def __init__(self) -> None:
        with open("./data/gen4/gen4_dex.json", "r", encoding="utf8") as file:
            self.pokedex = json.load(file)
        with open("./data/gen4/gen4_abilities_dex.json", "r", encoding="utf8") as file:
            self.ability_dex = json.load(file)
        with open("./data/gen4/gen4_moves_dex.json", "r", encoding="utf8") as file:
            self.moves_dex = json.load(file)
        with open("./data/gen4/gen4_items_dex.json", "r", encoding="utf8") as file:
            self.item_dex = json.load(file)
        with open("./data/gen4/gen4_types_dex.json", "r", encoding="utf8") as file:
            self.type_dex = json.load(file)
        with open("./data/gen4/natures.json", "r", encoding="utf8") as file:
            self.natures_dex = json.load(file)

        self.id_to_move_dict = {}
        self.move_to_id_dict = {}

        for i, x in enumerate(self.moves_dex):
            self.id_to_move_dict[i] = x
            self.move_to_id_dict[x] = i

        move_num = len(self.id_to_move_dict)
        for i, mon in enumerate(self.pokedex):
            self.id_to_move_dict[i + move_num] = "switch " + mon
            self.move_to_id_dict["switch " + mon] = i + move_num

    def parse_stats(self, line, stat_dict):
        ev_line = line.split()[1:][::-1]
        ev_line = [value for value in ev_line if value != "/"]
        for i in range(int(len(ev_line) / 2)):
            stat_dict[ev_line[2 * i].lower()] = int(ev_line[2 * i + 1])
        return stat_dict

    def calc_hp_stat(self, base, iv, ev, level):
        return (
            math.floor((2 * base + iv + math.floor(ev / 4)) * level / 100) + level + 10
        )

    def calc_non_hp_stat(self, base, iv, ev, level, nature, stat):
        nature_mod = 1
        if (
            "plus" in self.natures_dex[nature]
            and stat == self.natures_dex[nature]["plus"]
        ):
            nature_mod = 1.1
        elif (
            "minus" in self.natures_dex[nature]
            and stat == self.natures_dex[nature]["minus"]
        ):
            nature_mod = 0.9

        return math.floor(
            (math.floor((2 * base + iv + math.floor(ev / 4)) * level / 100) + 5)
            * nature_mod
        )

    def import_team(self, file):
        with open(file, "r") as fp:
            team = fp.read()

        team = team.split("\n")
        team_dict = {}

        mon = None
        for line in team:
            if "@" in line:
                mon_name = line[: line.find("@") - 1]
                if "(" in mon_name:
                    mon_name = mon_name[: mon_name.find("(") - 1]
                elif "-" in mon_name and "Rotom" not in mon_name:
                    mon_name = mon_name[: mon_name.find("-")]
                mon_name = normalize_name(mon_name)
                team_dict[mon_name] = defaultdict(dict)

                mon = team_dict[mon_name]

                mon["level"] = 100
                mon["types"] = [x.lower() for x in self.pokedex[mon_name]["types"]]
                mon["evs"] = {"hp": 0, "atk": 0, "def": 0, "spa": 0, "spd": 0, "spe": 0}
                mon["ivs"] = {
                    "hp": 31,
                    "atk": 31,
                    "def": 31,
                    "spa": 31,
                    "spd": 31,
                    "spe": 31,
                }
                mon["moves"] = []
                mon["item"] = normalize_name(line[line.find("@") + 2 :])
            elif "Ability:" in line:
                if mon is None:
                    raise ValueError("Ability line found before Pokemon line")

                mon["ability"] = normalize_name(line[9:])

            elif "EVs" in line:
                if mon is None:
                    raise ValueError("EVs line found before Pokemon line")

                mon["evs"] = self.parse_stats(line, mon["evs"])

            elif "IVs" in line:
                if mon is None:
                    raise ValueError("IVs line found before Pokemon line")

                mon["ivs"] = self.parse_stats(line, mon["ivs"])

            elif "-" not in line and "Nature" in line:
                if mon is None:
                    raise ValueError("Nature line found before Pokemon line")

                mon["nature"] = normalize_name(line.split()[0])

            elif len(line) == 0:
                continue

            elif "-" == line[0]:
                if mon is None:
                    raise ValueError("- line found before Pokemon line")

                move = normalize_name(line[2:])
                mon["moves"].append(
                    Move(
                        id=move,
                        disabled=False,
                        pp=int(self.moves_dex[move]["pp"] * 1.6),
                    )
                )

        poke_dict = {}
        for mon_name in team_dict:
            hp_stat = self.calc_hp_stat(
                self.pokedex[mon_name]["baseStats"]["hp"],
                team_dict[mon_name]["ivs"]["hp"],
                team_dict[mon_name]["evs"]["hp"],
                team_dict[mon_name]["level"],
            )
            poke_dict[mon_name] = Pokemon(
                id=mon_name,
                level=team_dict[mon_name]["level"],
                types=team_dict[mon_name]["types"],
                hp=hp_stat,
                maxhp=hp_stat,
                ability=team_dict[mon_name]["ability"],
                item=team_dict[mon_name]["item"],
                attack=self.calc_non_hp_stat(
                    self.pokedex[mon_name]["baseStats"]["atk"],
                    team_dict[mon_name]["ivs"]["atk"],
                    team_dict[mon_name]["evs"]["atk"],
                    team_dict[mon_name]["level"],
                    team_dict[mon_name]["nature"],
                    "atk",
                ),
                defense=self.calc_non_hp_stat(
                    self.pokedex[mon_name]["baseStats"]["def"],
                    team_dict[mon_name]["ivs"]["def"],
                    team_dict[mon_name]["evs"]["def"],
                    team_dict[mon_name]["level"],
                    team_dict[mon_name]["nature"],
                    "def",
                ),
                special_attack=self.calc_non_hp_stat(
                    self.pokedex[mon_name]["baseStats"]["spa"],
                    team_dict[mon_name]["ivs"]["spa"],
                    team_dict[mon_name]["evs"]["spa"],
                    team_dict[mon_name]["level"],
                    team_dict[mon_name]["nature"],
                    "spa",
                ),
                special_defense=self.calc_non_hp_stat(
                    self.pokedex[mon_name]["baseStats"]["spd"],
                    team_dict[mon_name]["ivs"]["spd"],
                    team_dict[mon_name]["evs"]["spd"],
                    team_dict[mon_name]["level"],
                    team_dict[mon_name]["nature"],
                    "spd",
                ),
                speed=self.calc_non_hp_stat(
                    self.pokedex[mon_name]["baseStats"]["spe"],
                    team_dict[mon_name]["ivs"]["spe"],
                    team_dict[mon_name]["evs"]["spe"],
                    team_dict[mon_name]["level"],
                    team_dict[mon_name]["nature"],
                    "spe",
                ),
                moves=team_dict[mon_name]["moves"],
                # nature not supported by poke-engine
                # nature=team_dict[mon_name]['nature'],
            )

        side_dict = {
            "wish": (0, 0),
            "side_conditions": defaultdict(int),
            "future_sight": (0, 0),
            "reserve": {},
        }

        for i, mon in enumerate(poke_dict):
            if i == 0:
                side_dict["active"] = poke_dict[mon]
            else:
                side_dict["reserve"][mon] = poke_dict[mon]

        side = Side(
            pokemon=list(poke_dict.values()),
            wish=(0, 0),
            side_conditions=SideConditions(),
        )

        return side

    def initialize_state(self, file1, file2):
        state = State(
            side_one=self.import_team(file1),
            side_two=self.import_team(file2),
            weather="none",
            terrain="none",
            trick_room=False,
        )

        return state
