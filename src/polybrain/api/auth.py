"""

Handles auth related things

"""


from textwrap import dedent
import dotenv
from pathlib import Path
from loguru import logger

class ApiAuthManager:

    def __init__(self) -> None:
        self.cert_file, self.key_file = self.load_ssl()

    def load_ssl(self) -> tuple[Path, Path]:
        """Loads SSL certificates
        
        Returns:
            A tuple of the (cert_file, key_file)
        """

        env_file = Path(dotenv.find_dotenv())

        ssl_path = dotenv.get_key(env_file, "SSL_PATH")

        if ssl_path is None:
            logger.error(dedent(
                """
                \n\nTo run the API, you need an SSL key.
                Run the following to generate an SSL key:

                openssl req -x509 -newkey rsa:4096 -keyout self-signed.key -out self-signed.crt -sha256 -days 36500 -nodes -subj "/C=US/ST=NY/O=Polybrain/OU=Polybrain/CN=localhost" -addext "subjectAltName = DNS:localhost"

                ssl keys. Save these to a safe directory, and then store the path of this directory in the `SSL_PATH`
                field of your .env file. Then, re-run this script.
                If you are using chrome, you may need to turn the #allow-insecure-localhost flag to true as well.

                """
            ))
            exit(1)


        ssl_path_abs = Path.absolute(Path(ssl_path))

        cert_file = Path.joinpath(ssl_path_abs, "cert.pem")
        key_file = Path.joinpath(ssl_path_abs, "key.pem")

        return cert_file, key_file

