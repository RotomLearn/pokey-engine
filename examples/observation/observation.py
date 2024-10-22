from pokey_engine import State, Move, Pokemon, SideConditions
from pokey_engine.pokezoo import observations
from utilities import Utilities

if __name__ == "__main__":
    state = Utilities().initialize_state("./team1.txt", "./team2.txt")

    print(state)

    move = Move(
        id="tackle",
        disabled=False,
        pp=int(32),
        move_num=int(5),
    )

    print(move)

    pokemon = Pokemon("lapras", types=["water"])
    print(pokemon)

    side_conditions = SideConditions()
    print(side_conditions)

    print(observations(state))

    print(state.battle_is_over())

    all_options = state.get_all_options()
    for side in all_options:
        print(side)

    instructions = state.generate_instructions(
        str(all_options[0][0]), str(all_options[1][3])
    )

    for i in instructions:
        print(i.percentage)
        print(i.instruction_list)

    state.apply_instructions(1)
    print(state)
