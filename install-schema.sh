cp -f ./data/schema/org.hypr.Hyprmaster.tablet.gschema.xml \
  /usr/share/glib-2.0/schemas

glib-compile-schemas /usr/share/glib-2.0/schemas
