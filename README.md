# Observability Statistics for transparent Route Servers

To build this run:

```cargo build --release```

and then run:

```./target/release/aspa_transparent_rses```

, which will obtain statistics for the Route Server ASNs and timestamp hard-coded in  ```src/main.rs```. If you change the timestamp, please make sure that you pick only timestamps that represent 00:00 UTC+0, 08:00 UTC+0, or 16:00 UTC+0 of any past day, as these are the times for which all collectors produce RIB snapshots.




###  Results for 2023-07-24, 16:00:00 UTC
```
IX                   | RS ASN   | re-appending members | IPv4 Prefixes involved    | IPv6 Prefixes involved   
---------------------+----------+----------------------+---------------------------+--------------------------
HKIX                 | 4635     | 42                   | 182392                    | 48864                    
TorIX                | 11670    | 4                    | 3635                      | 3                        
DE-CIX Frankfurt     | 6695     | 4                    | 85105                     | 6669                     
LINX                 | 8714     | 3                    | 4486                      | 165                      
AMS-IX               | 6777     | 2                    | 1873                      | 395                      
NIC.CZ               | 47200    | 1                    | 79                        | 0                        
IX.br                | 26162    | 1                    | 10                        | 0                        
France-IX Paris      | 51706    | 1                    | 41                        | 0                        
Netnod               | 52005    | 1                    | 27                        | 0                        
NapAfrica            | 37195    | 0                    | 0                         | 0                        
MSK-IX Moscow        | 8631     | 0                    | 0                         | 0                        
Any2                 | 32184    | 0                    | 0                         | 0                        
MIX-IT               | 61968    | 0                    | 0                         | 0                        
SIX                  | 33108    | 0                    | 0                         | 0                        
NL-ix                | 34307    | 0                    | 0                         | 0                        
PiterIX              | 49869    | 0                    | 0                         | 0                        
SwissIX              | 42476    | 0                    | 0                         | 0                        
NYIIX                | 13538    | 0                    | 0                         | 0                        
DATAIX               | 50952    | 0                    | 0                         | 0                        
DTEL-IX              | 31210    | 0                    | 0                         | 0                        
DE-CIX New York      | 63034    | 0                    | 0                         | 0       
```
