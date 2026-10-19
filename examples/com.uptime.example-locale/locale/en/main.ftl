# Every message is this pack's prefix plus a name with no dash.
# The pack id com.uptime.example-locale gives mod-com-uptime-example--locale-
# (each dot becomes one dash, each dash becomes two).
# Terms such as -pack-name stay inside this pack.

-pack-name = Night Shift

mod-com-uptime-example--locale-title = { -pack-name }
mod-com-uptime-example--locale-greeting = Welcome to the night shift.
    .eng = On call. Pager is live.
mod-com-uptime-example--locale-tickets = { $count ->
    [one] One ticket waits for you.
   *[other] { $count } tickets wait for you.
}
