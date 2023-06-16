# Automation powered by CronCat

Cronkitty is a plugin that allows for [Vectis] users to use [CronCat] seamlessly.
To understand more about how CronCat brings automation to CosmWasm enabled chains without protocol level update,
please see their [detailed docs].

## Features

[CronCat] is a powerful automation infrastructure,
this plugin enhances the user experience by providing 3 distinctive features via a fine-grained permission design.

-   Auto-refill: never worry about your automation task running out of credit on [CronCat].
    Cronkitty allows the Vectis account to auto refill tasks based on the remaining credit amount.
    Users can also decide when to stop the top up.
-   Self-custody: Cronkitty acts as a authorised party to execute on the Vectis account.
    Funds never leave your Vectis account until it is called by CronCat -- Cronkitty -- your wallet!
-   Preserve sender origin: Instead of the target application thinking that the sender of an instruction is from CronCat,
    using Cronkitty + Vectis Account allows sender to always be your wallet, even if it was automated

[vectis]: https://www.vectis.space/
[croncat]: https://cron.cat
[detailed docs]: https://docs.cron.cat
