# Glossary

These words have specific meanings throughout this specification.

## Item

One of the strings or schemas that a [key] is computed from, in order. The
application chooses the items. In the `postcard-schema` crate, an item
is a `key::Item`. See [What the Application Chooses][chooses].

## Key

The 8-byte result of hashing the [key input]. See [Keys][keys].

## Key Input

The bytes hashed into a [key]: a header (the domain, the [options] and the item
count), then the [items][item]. See [Key Input][key-input-section].

## Options

The settings that decide whether the names of fields and variants, and maximum
lengths, are part of the [key input]. The application chooses the options.
See [What the Application Chooses][chooses].

## Schema Table

One or more schemas, stored as a list of types and one string of names, with
each type and name stored once. It's how schemas are sent. See
[Schema Tables][tables].

[item]: #glossary--item
[items]: #glossary--item
[key]: #glossary--key
[key input]: #glossary--key-input
[options]: #glossary--options
[keys]: #keys
[tables]: #schema-tables
[chooses]: #keys--how-keys-work--what-the-application-chooses
[key-input-section]: #keys--key-input
