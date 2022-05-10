//-------------------------------------------------
// This is needed for "enabling jasmine for ts"
const TSConsoleReporter = require("jasmine-ts-console-reporter");
jasmine.getEnv().clearReporters(); //Clear default console reporter
jasmine.getEnv().addReporter(new TSConsoleReporter());
// This is high because each suite may take some time and if it is too low
// the suite will time out and start the next suite without warning.
jasmine.DEFAULT_TIMEOUT_INTERVAL = 600*1000;
