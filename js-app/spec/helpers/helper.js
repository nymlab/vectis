//-------------------------------------------------
// This is needed for "enabling jasmine for ts"
const TSConsoleReporter = require("jasmine-ts-console-reporter");
jasmine.getEnv().clearReporters(); //Clear default console reporter
jasmine.getEnv().addReporter(new TSConsoleReporter());
jasmine.getEnv().configure({random: false});
jasmine.DEFAULT_TIMEOUT_INTERVAL = 60*1000;
