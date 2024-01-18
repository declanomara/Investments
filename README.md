# O'Mara Investments
## About
This project is a collection of tools for trading forex. It is written in Rust to ensure stability and speed, two obviously critical factors in trading. It is designed to be modular, so that it can be easily extended to support other markets and trading strategies. It is currently in the early stages of development, and is not designed for public use, but rather to allow me to learn more about trading and Rust, and hopefully make some money in the process.

## Structure
This project contains the following crates:
- `quantlib`: The main crate, containing the shared code for the project.
- `trading`: The trading bot, which is used to trade forex.
- `research`: The research crate, used to research trading strategies through backtesting.
- `data-collection`: The data crate, which will be used to download and store data.

## Status/Roadmap
Currently, the project is in the early stages of development. Basic examples of all four aspects of the project have been implemented, but they are not yet integrated. The next steps are:

1. Streamline the research crate. I got too excited and tried to generalize too much to my own demise. For the time being, rewriting the research crate while developing new strategies is fine, but once I have a strategy that I am confident in, I will need to rewrite the research crate to be more general.
2. Move all definitions of shared types to the `quantlib` crate. (Currently the `research` crate has its own definitions of types that are also defined in `quantlib`.)

Once those are done I can begin to forward test strategies on a demo account in the AWS cloud. The data-collection crate seems quite stable, I have successfully collected data since October 2023 without any human involvement, and I have been using that data to try to develop strategies. Once I am confident that the system is stable and profitable, I can begin to trade with real money, and focus on rewriting production versions of the crates to be more profitable through measures such as minimizing latency and fancier optimization techniques.
