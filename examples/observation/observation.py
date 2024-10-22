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
