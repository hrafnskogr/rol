/* Error module
 * Custom Error definition
 */

use std::fmt;

pub struct ROLErr
{
    pub code: usize,
    pub message: String,
    pub wsa_code: usize,
}

impl fmt::Display for ROLErr
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        let msg = match self.code
        {
            1 => String::from("MAC address format invalid.\n Please make sure it is of the following type: 00:AA:BB:CC:DD:EE or 00AABBCCDDEEFF - case insensitive."),
            2 => String::from("MAC address too short\n Please make sure it is of the following type: 00:AA:BB:CC:DD:EE or 00AABBCCDDEEFF - case insensitive."),
            3 => String::from("MAC address too long\n Please make sure it is of the following type: 00:AA:BB:CC:DD:EE or 00AABBCCDDEEFF - case insensitive."),
            4 => format!("WSA Error - Please refer to the following code: {}", self.wsa_code),
            _ => String::from("Something went wrong."),
        };

        write!(f, "{}", msg)
    }
}

impl fmt::Debug for ROLErr
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        write!(f, "ROLErr:\n{{\n\tCode: {}\n\tMessage: {}\n\tWSA Code: {}\n}}", self.code, self.message, self.wsa_code)
    }
}
