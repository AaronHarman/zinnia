# ZINNIA
## Zero Internet Necessary Non-Intelligent Assistant

ZINNIA is a voice assistant that does not use any generative AI, and all of the voice processing, wakeword detection, and voice synthesis is done locally on the user's device.
Some functionalities of ZINNIA do require access to the internet, such as getting the current weather.

This software is very much a work in progress. Current problems include excessive memory usage, slow startup time, and a low number of commands. I'm considering reworking it to be hosted on a local server to allow the client part of the software to have a smaller memory footprint.

Current functionality includes:
- Commands to roll dice, set a timer, check the weather, tell a joke, explain a command (help)
- Wakeword detection, ZINNIA begins listening when it hears "Yo, ZINNIA"
- Speech synthesis and system notifications for ZINNIA's responses

Currently you can attempt to build ZINNIA on your machine at your own risk. It has only been tested on a Ubuntu machine, and likely is missing necessary files in the repo in order to build. A downloadable build will be released once the software is more stable.
