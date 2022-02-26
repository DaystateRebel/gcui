# gcui
Open source tool to program Daystate V5 GCUs

Gcui is able to up and download the power settings from and to a Daystate GCU v5 as found in the Red Wolf. Gcui has the following command line interface

To read power settings from the GCU 

`$ gcui --read --filename=<file>`

To write power settings to the GCU 

`$ gcui --write --filename=<file>`

To read the current air pressure from the GCU

`$ gcui --pressure`

To read the current pulse duration from the GCU

`$ gcui --pulse`

To read the GCU version

`$ gcui --rwversion`
