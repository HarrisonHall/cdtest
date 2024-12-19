# cdtest

Simple tool to quickly traverse and manage semi-temporary test directories. By
default, test directories are stored in `/var/tmp/cdtest` and are
garbage-collected when `cdtest` is run again after being unused for 14 days.
