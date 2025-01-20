pub fn get_mock_sensor_data() -> String {
    r#"sensors
    tg3-pci-0300
    Adapter: PCI adapter
    temp1:        +55.0°C  (high = +100.0°C, crit = +110.0°C)
    
    coretemp-isa-0002
    Adapter: ISA adapter
    Package id 2:  +29.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 0:        +19.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 1:        +23.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 2:        +18.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 3:        +26.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 4:        +25.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 8:        +18.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 9:        +22.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 10:       +23.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 11:       +30.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 12:       +23.0°C  (high = +76.0°C, crit = +86.0°C)
    
    coretemp-isa-0000
    Adapter: ISA adapter
    Package id 0:  +29.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 0:        +22.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 1:        +25.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 2:        +25.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 3:        +24.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 4:        +24.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 8:        +24.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 9:        +28.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 10:       +25.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 11:       +25.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 12:       +27.0°C  (high = +76.0°C, crit = +86.0°C)
    
    acpitz-acpi-0
    Adapter: ACPI interface
    temp1:         +8.3°C
    
    coretemp-isa-0003
    Adapter: ISA adapter
    Package id 3:  +33.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 0:        +26.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 1:        +29.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 2:        +25.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 3:        +30.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 4:        +32.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 8:        +29.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 9:        +29.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 10:       +28.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 11:       +31.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 12:       +27.0°C  (high = +76.0°C, crit = +86.0°C)
    
    coretemp-isa-0001
    Adapter: ISA adapter
    Package id 1:  +31.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 0:        +25.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 1:        +19.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 2:        +28.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 3:        +27.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 4:        +25.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 8:        +26.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 9:        +28.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 10:       +29.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 11:       +32.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 12:       +23.0°C  (high = +76.0°C, crit = +86.0°C)
    
    power_meter-acpi-0
    Adapter: ACPI interface
    power1:      130.00 W  (interval = 300.00 s)
    
    root@edgeville:~# ^C
    root@edgeville:~# ^C
    root@edgeville:~# ^C
    root@edgeville:~# ^C
    root@edgeville:~# ^C
    root@edgeville:~# sensors
    tg3-pci-0300
    Adapter: PCI adapter
    temp1:        +55.0°C  (high = +100.0°C, crit = +110.0°C)
    
    coretemp-isa-0002
    Adapter: ISA adapter
    Package id 2:  +30.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 0:        +20.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 1:        +24.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 2:        +20.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 3:        +27.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 4:        +27.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 8:        +20.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 9:        +23.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 10:       +23.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 11:       +30.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 12:       +24.0°C  (high = +76.0°C, crit = +86.0°C)
    
    coretemp-isa-0000
    Adapter: ISA adapter
    Package id 0:  +28.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 0:        +23.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 1:        +26.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 2:        +27.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 3:        +25.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 4:        +26.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 8:        +24.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 9:        +27.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 10:       +27.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 11:       +26.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 12:       +26.0°C  (high = +76.0°C, crit = +86.0°C)
    
    acpitz-acpi-0
    Adapter: ACPI interface
    temp1:         +8.3°C
    
    coretemp-isa-0003
    Adapter: ISA adapter
    Package id 3:  +33.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 0:        +27.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 1:        +28.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 2:        +23.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 3:        +29.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 4:        +32.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 8:        +29.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 9:        +28.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 10:       +27.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 11:       +29.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 12:       +27.0°C  (high = +76.0°C, crit = +86.0°C)
    
    coretemp-isa-0001
    Adapter: ISA adapter
    Package id 1:  +32.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 0:        +25.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 1:        +20.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 2:        +29.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 3:        +26.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 4:        +26.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 8:        +28.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 9:        +30.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 10:       +28.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 11:       +32.0°C  (high = +76.0°C, crit = +86.0°C)
    Core 12:       +23.0°C  (high = +76.0°C, crit = +86.0°C)
    
    power_meter-acpi-0
    Adapter: ACPI interface
    power1:      129.00 W  (interval = 300.00 s)
    "#
    .to_string()
}
