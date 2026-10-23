#include "postcard.h"
#include <stdint.h>
#include <stdio.h>
#include <string.h>

/// helper function that prints out a byte buffer to stdout
void print_buffer(uint8_t* buffer, size_t len)
{
    printf("serialized data [");
    for (size_t i = 0; i < len; i++) {
        printf("%d", buffer[i]);
        if (i < len - 1) {
            printf(", ");
        }
    }
    printf("]\n");
}

/// helper function that prints out an array of int16 values to stdout
void print_values(int16_t* values, size_t len)
{
    printf("values: [");
    for (size_t i = 0; i < len; i++) {
        printf("%d", values[i]);
        if (i < len - 1) {
            printf(", ");
        }
    }
    printf("]\n");
}

int main()
{

    // This example shows manually serializing and deserializing the Foo struct below.
    //
    // ```rust
    // struct Foo {
    //     id: u32,
    //     name: String,
    //     values: Vec<i16> // len 3
    //     is_active: bool
    // }
    // ```

    // allocate a buffer large enough to fit the serialized data
    uint8_t buffer[128];

    // create a new `postcard_slice_t` that uses `buffer` as the underlying storage
    // `postcard_slice_t` is a growable reference to some underlying buffer.
    postcard_slice_t slice;
    postcard_init_slice(&slice, buffer, sizeof(buffer));

    // encode id
    postcard_encode_u32(&slice, 1234);

    // encode name
    const char* name = "PostcardTest";
    postcard_encode_string(&slice, name, strlen(name));

    // encode the 3 values
    postcard_start_seq(&slice, 3);
    postcard_encode_i16(&slice, -10);
    postcard_encode_i16(&slice, 20);
    postcard_encode_i16(&slice, -30);

    // encode is_active
    postcard_encode_bool(&slice, true);

    // print the encoded data
    // slice.len now contains the length of the serialized
    // data from the serialization function
    print_buffer(slice.data, slice.len);

    // to decode the data we will create a new slice
    // in the decode path, `postcard_slice_t.len` is used
    // as a cursor for the decoded data. So we need a new slice
    // that will only reference the serialized data, and
    // have len = 0
    postcard_slice_t decode_slice;
    postcard_init_slice(&decode_slice, buffer, slice.len);

    // decode id
    uint32_t id;
    postcard_decode_u32(&decode_slice, &id);
    printf("id: %u\n", id);

    // decode name
    size_t actual_len;
    // first we decode the length of the string
    // this is pulled out as a different function,
    // so you can heap-allocate a string based on the actual
    // length of the string
    postcard_decode_string_len(&decode_slice, &actual_len);
    char name_buffer[actual_len + 1];
    postcard_decode_string(&decode_slice, name_buffer, sizeof(name_buffer),
        actual_len);
    name_buffer[actual_len] = '\0'; // null terminate the string
    // strings in postcard are encoded as byte arrays of valid utf8 data,
    // and then do not include a null terminator. We add this terminator
    // manually so we can print the string using `printf`
    printf("name: %s\n", name_buffer);

    // decode values
    size_t seq_len;
    postcard_decode_seq_len(&decode_slice, &seq_len);
    printf("values len: %zu\n", seq_len);

    int16_t values[seq_len];
    for (size_t i = 0; i < seq_len; i++) {
        postcard_decode_i16(&decode_slice, &values[i]);
    }
    print_values(values, seq_len);

    // decode is_active
    bool is_active;
    postcard_decode_bool(&decode_slice, &is_active);
    printf("is_active: %s\n", is_active ? "true" : "false");

    return 0;
}
