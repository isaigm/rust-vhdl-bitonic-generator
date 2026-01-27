library IEEE;
use IEEE.STD_LOGIC_1164.ALL;
use IEEE.NUMERIC_STD.ALL;

entity push_btn is
    Port ( 
        CLK100MHZ : in  std_logic; 
        btn      : in  std_logic; 
        enabled   : out std_logic
    );
end push_btn;

architecture Behavioral of push_btn is
    signal s_enabled : std_logic := '0';
    signal btn_sync_0 : std_logic := '0'; 
    signal btn_sync_1 : std_logic := '0'; 
    signal btn_last   : std_logic := '0'; 
begin

    process (CLK100MHZ)
    begin
        if rising_edge(CLK100MHZ) then
            btn_sync_0 <= btn;     
            btn_sync_1 <= btn_sync_0; 
            btn_last   <= btn_sync_1;
            s_enabled <= '0'; 
            if btn_sync_1 = '1' and btn_last = '0' then
                s_enabled <= '1';
            end if;
            
        end if;
    end process;

    enabled <= s_enabled;

end Behavioral;