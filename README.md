# O'Mara Investments
## About
This project is a collection of tools for trading forex. It is written in Rust to ensure stability and speed, two obviously critical factors in trading. It is designed to be modular, so that it can be easily extended to support other markets and trading strategies. It is currently in the early stages of development, and is not designed for public use, but rather to allow me to learn more about trading and Rust, and hopefully make some money in the process.

## Structure
This project contains the following crates:
- `quantlib`: The main crate, containing the shared code for the project.
- `trading`: The trading bot, which is used to trade forex.
- `research`: The research crate, used to research trading strategies through backtesting.
- `(planned) data`: The data crate, which will be used to download and store data.

## Status/Roadmap
Currently, the project is in the early stages of development. All three of the crates that have been implemented so far are incredibly primitive and basically just prototypes. My approach to development is to first implement a minimal viable product, and then with that experience I can more easily and properly design a production version. I am still very much in the prototyping phase and do not have a minimal viable product yet, but I am getting close. The current priorities are:

1. Implement a genetic algorithm to optimize trading strategies.
2. Move all definitions of shared types to the `quantlib` crate. (Currently the `research` crate has its own definitions of types that are also defined in `quantlib`.)
3. Implement a minimal viable product for the `data` crate.
4. Implement some sort of logging system.

Once those are done I can begin to forward test strategies on a demo account in the AWS cloud. I anticipate that there will be many issues with the prototype once it is running, and as I discover those issue I can redesign the system to be more robust. Once I am confident that the system is stable and profitable, I can begin to trade with real money, and focus on rewriting production versions of the crates to be more profitable through measures such as minimizing latency and optimizing the genetic algorithm.
