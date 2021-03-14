import logo from './logo.svg';
import Button from '@material-ui/core/Button'
import Table from '@material-ui/core/Table';
import TableBody from '@material-ui/core/TableBody';
import TableCell from '@material-ui/core/TableCell';
import TableHead from '@material-ui/core/TableHead';
import TableRow from '@material-ui/core/TableRow';
import TableContainer from '@material-ui/core/TableContainer';
import Paper from '@material-ui/core/Paper';
import { useState, useEffect } from 'react';
import AppBar from '@material-ui/core/AppBar';
import CircularProgress from '@material-ui/core/CircularProgress';
import Toolbar from '@material-ui/core/Toolbar';
import './App.css';
import Alert from '@material-ui/lab/Alert';

// Make debugging between node and rust server easier
// const API_URL = "http://localhost:8000/"
const API_URL = "" // For release

interface DeviceState {
  voltage: number,
  current: number,
  power: boolean,
  error: boolean,
}

interface DeviceMap {
  [name: string]: DeviceState,
}

function PowerButton(props: { DevName: string; PowerState: boolean; updateData: (arg0: DeviceMap) => void }) {
  let [isDisabled, setisDisabled] = useState(false);

  let Clicked = () => {
    setisDisabled(true);
    console.log(`Toggling ${props.DevName}`);

    fetch(`${API_URL}device/${props.DevName}/toggle`)
      .then(response => {
        if (response.ok) {
          return response.json()
        }
        else {
          return Promise.reject("Toggle Failed")
        }
      })
      .then((data: DeviceState) => {
        props.updateData({ [props.DevName]: data });
      })
      .catch((err) => {
        alert(`Failure: ${err}`);
        console.error(err)
      })
      .then(() => { setisDisabled(false) });
  };

  // TODO - This used to be a catch-all. 
  // Pretty sure its unnecessary since the promise is implemented 
  // useEffect(() => {
  //   setisDisabled(false);
  // }, [props]); 

  return <Button variant="contained"
    style={{ backgroundColor: props.PowerState ? "limegreen" : "red" }}
    disabled={isDisabled}
    onClick={Clicked} >
    {props.PowerState ? "On" : "Off"}
  </Button>
}

function App() {

  let [rows, setrowsdata] = useState<DeviceMap>({});
  let [pendingData, setpendingData] = useState(false);
  let [wasFailure, setwasFailure] = useState(false);

  function updateData(myRows: DeviceMap) {
    setrowsdata(Object.assign({}, rows, myRows));
  }

  function requestUpdate() {
    setpendingData(true);

    fetch(`${API_URL}device`)
      .then(z => z.json())
      .then((data: DeviceMap) => { updateData(data); setwasFailure(false) })
      .catch(err => { console.error(err); setwasFailure(true) })
      .then(() => { setpendingData(false) });
  }

  useEffect(() => {
    requestUpdate();
    const interval = setInterval(requestUpdate, 2000);
    return () => clearInterval(interval);
    // Only care about DidMount so ignore warning about deps. This will never be called again
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <div className="App">
      <AppBar >
        <Toolbar style={{ gap: "10px" }}>
          <img src={logo} className="App-logo" alt="logo" height={40} />
          Koradmote
          {wasFailure && <Alert variant="filled" severity="error">Last Update Request Failed</Alert>}
          {pendingData && <CircularProgress disableShrink color="secondary" />}
          {/* <Fab color="secondary" aria-label="add" style={{ margin: 0, right: 20, position: 'fixed' }}
          onClick={requestUpdate}>
            { <RefreshIcon /> }
          </Fab> */}
        </Toolbar>
      </AppBar>
      <Toolbar />
      <div className="Table">
        <TableContainer component={Paper}>
          <Table className={"JackTest"} aria-label="simple table">
            <TableHead>
              <TableRow>
                <TableCell style={{ fontWeight: 900 }}>Name</TableCell>
                <TableCell style={{ fontWeight: 900 }}>Voltage</TableCell>
                <TableCell style={{ fontWeight: 900 }}>Current</TableCell>
                <TableCell style={{ fontWeight: 900 }}>Current State</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {
                Object.entries(rows).map(([key, value]) =>
                  <TableRow key={key}>
                    <TableCell component="th" scope="row">{key}</TableCell>
                    <TableCell>{value.voltage.toFixed(3)}</TableCell>
                    <TableCell>{value.current.toFixed(3)}</TableCell>
                    <TableCell>{value.error &&
                      <Alert variant="filled" severity="error" style={{width:"max-content"}}>Last Update Failed</Alert>
                      || <PowerButton DevName={key} PowerState={value.power} updateData={updateData}></PowerButton>}</TableCell>
                  </TableRow>
                )}
            </TableBody>
          </Table>
        </TableContainer>
      </div>
    </div>
  );
}

export default App;
