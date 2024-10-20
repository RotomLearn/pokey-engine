from pokey_engine import State, Move, Pokemon

if __name__ == "__main__":
    state = State()
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
