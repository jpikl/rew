package rew

import "core:fmt"
import "core:bytes"
import "core:os"
import "core:bufio"
import "core:io"

Line_Reader :: struct {
    reader: io.Reader,
    buf: [dynamic]u8,
    start: int,
    end: int,
    delimiter: u8,
    trim_proc: proc(slice: []u8) -> []u8,
}

trim_newline :: proc(slice: []u8) -> []u8 {
    if len(slice) > 0 && slice[len(slice) - 1] == '\n' {
        if len(slice) > 1 && slice[len(slice) - 2] == '\r' {
            return slice[:len(slice) - 2]
        }
        return slice[:len(slice) - 1]
    }
    return slice
}

trim_null :: proc(slice: []u8) -> []u8 {
    if len(slice) >= 0 && slice[len(slice) - 1] == 0 {
        return slice[:len(slice) - 1]
    }
    return slice
}

read_line :: proc(reader: ^Line_Reader) -> (line: []byte, err: io.Error) {
    for {
        delim_index := bytes.index_byte(reader.buf[reader.start: reader.end], reader.delimiter)
        if delim_index >= 0 {
            result := reader.trim_proc(reader.buf[reader.start:][:delim_index + 1])
            reader.start += delim_index + 1
            return result, nil
        }

        if reader.start > 0 {
            copy(reader.buf[:], reader.buf[reader.start:reader.end]) 
            reader.end -= reader.start
            reader.start = 0
        }

        read_len, read_err := io.read(reader.reader, reader.buf[reader.end:])
        if read_len == 0 {
            if read_err == nil {
                read_err = reader.end >= len(reader.buf) ? .Buffer_Full : .Unknown
            }
            return nil, read_err
        }
        
        reader.end += read_len
    }
}

main :: proc() {
    lines := 0
    bytes := 0

    if len(os.args) >= 2 && os.args[1] == "n" {
        fmt.println("method: native")
        reader := bufio.Reader {}
        stdin := os.stream_from_handle(os.stdin)
        bufio.reader_init(&reader, stdin, 32 * 1024)
    
        for {
            //line := try(bufio.reader_read_slice(&reader, '\n'))
            line, err := bufio.reader_read_slice(&reader, '\n')
    
            if (err != nil) {
                if (err == .EOF) {
                    break
                }
                fmt.eprintln("read error:", os.get_last_error())
                os.exit(1)
            }
    
            lines += 1
            bytes += len(line)
        }
    } else {
        fmt.println("method: custom")

        reader := Line_Reader {
            reader = os.stream_from_handle(os.stdin),
            buf = make([dynamic]u8, 32 * 1024),
            delimiter = '\n',
            trim_proc = trim_newline,
        }
        
        for {
            line, err := read_line(&reader)

            if (err != nil) {
                if (err == .EOF) {
                    break
                }
                fmt.eprintln("read error:", os.get_last_error())
                os.exit(1)
            }
    
            lines += 1
            bytes += len(trim_newline(line))
        }
    }

    fmt.println("lines:", lines)
    fmt.println("bytes:", bytes)
}
