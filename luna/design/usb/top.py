# - transfer

transfer: Transfer

transaction: Transaction 1
transaction_2: Transaction 2
transaction_n: Transaction n

token_packet: Token Packet
token_packet: {
    shape: sql_table
    pid: (packet identifier)
    address
    endpoint
    crc
}

data_packet: Data Packet (optional)
data_packet: {
    shape: sql_table
    pid: (packet identifier)
    data
    crc
}

handshake_packet: Handshake Packet
handshake_packet: {
    shape: sql_table
    pid: (packet identifier)
}

transfer -> transaction
transfer -> transaction_2
transfer -> transaction_n
transaction -> token_packet
transaction -> data_packet
transaction -> handshake_packet

# - packet identifiers

packet_identifiers: Packet Identifiers
packet_identifiers.txt: |txt

Token Packet:

    OUT    0b0001  device address, endpoint address for OUT transaction
    IN     0b1001  device address, endpoint address for IN transaction
    SOF    0b0101  start-of-frame marker, frame number
    SETUP  0b1101  device address, endpoint address for SETUP transaction

Data Packet:

    DATA0  0b0011  data toggle or data PID sequencing
    DATA1  0b1011  data toggle or data PID sequencing
    DATA2  0b0111  data PID sequencing
    MDATA  0b1111  data PID sequencing

Status / Handshake:

    ACK    0b0010
    NAK    0b1010
    STALL  0b1110
    NYET   0b0110

Special:

    PRE    0b1100
    ERR    0b1100
    SPLIT  0b1000
    PING   0b0100
    EXT    0b0000
|

# - transaction

transactions: Transactions
transactions.txt: |txt
    - between endpoints
    - begins when host sends token packet containing:
        target_endpoint_number
        direction: OUT contains data from host, IN requests data from device
    -
|

endpoint: Endpoint
endpoint.txt: |c
    {
        buffer:    [u8; ?],
        address:   u4,
        direction: u1,      // Defined from host perspective:
                            //   0=OUT: host   - host sending data to device
                            //   1=IN:  device - host requesting data from device
    }
|

transactiontypes: Transaction Types
transactiontypes.txt: |txt
    CONTROL:
        stage 1: SETUP            -> RequestPacket
        stage 2: DATA (Optional)  -> [u8; <= 512, 64 or 8]   ?
        stage 3: STATUS           -> Success / Fail          ?
    BULK
        defined by class
    INTERRUPT
        defined by class
    ISOCHRONOUS
        defined by class
|

transactions: Transactions
transactions.txt: |txt
    : Type   : Source    : Used By    : Contents               :
    : ------ : --------- : ---------- : ---------------------- :
    : SETUP  : Host      : CONTROL    : SETUP (RequestPacket)  :
    : OUT    : Host      : all        : DATA or STATUS         :
    : IN     : Device    : all        : DATA or STATUS         :
|

# - sequence diagrams ---------------------------------------------------------

setup: {
    label: SETUP
    shape: sequence_diagram
    a: {
        label: Host
    }
    b: {
        label: Device
    }

    transaction_1: {
        a -> b: SET_ADDRESS
        a <- b: ACK
    }

    transaction_2: {
        a -> b: GET_STATUS
        a <- b: 0b1
    }
}
