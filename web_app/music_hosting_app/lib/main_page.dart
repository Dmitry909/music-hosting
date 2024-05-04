import 'package:flutter/material.dart';
import 'package:http/http.dart' as http;

import 'signup_page.dart';
import 'login_page.dart';
import 'shared_state.dart';

class MainPage extends StatefulWidget {
  @override
  _MainPageState createState() => _MainPageState();
}

class _MainPageState extends State<MainPage> {
  bool _isTokenValid = false;
  String username = "USERNAME INITIAL VALUE";

  @override
  void initState() {
    print('Inside initState');
    super.initState();
    print('Calling _checkTokenValidity');
    _checkTokenValidity();
  }

  Future<void> _checkTokenValidity() async {
    print('Called _checkTokenValidity');
    username = await getUsername() ?? "FAILED TO GET USERNAME";
    setState(() {
      _isTokenValid = false;
    });
    print('Called _checkTokenValidity 2');
    try {
      print('try');
      final token = (await getToken()) ?? "";
      print('token: ');
      print(token);
      print('TOKEN FINISH');
      if (token != "") {
        final response = await http.get(
            Uri.parse('http://localhost:3002/check_token'),
            headers: {'Authorization': token});
        print(response);
        if (response.statusCode == 200) {
          setState(() {
            _isTokenValid = true;
          });
        }
      }
    } catch (e) {
      print("fgh");
    }

    print(_isTokenValid);
  }

  Future<void> _logout() async {
    // TODO
    final token = (await getToken())!;
    storeToken(username, "");

    final response = await http.post(
      Uri.parse('http://localhost:3002/logout'),
      headers: {'authorization': token},
    );

    print(response);

    Navigator.pushReplacement(
        context,
        MaterialPageRoute(
          builder: (context) => MainPage(),
        ));
  }

  @override
  Widget build(BuildContext context) {
    print('Inside build MainWidget');
    return _isTokenValid
        ? _buildWelcomePage(context)
        : _buildLoginSignupPage(context);
  }

  Widget _buildLoginSignupPage(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('Main Page'),
      ),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            ElevatedButton(
              onPressed: () {
                Navigator.push(
                  context,
                  MaterialPageRoute(builder: (context) => LoginPage()),
                );
              },
              child: Text('Log in'),
            ),
            SizedBox(height: 16),
            ElevatedButton(
              onPressed: () {
                Navigator.push(
                  context,
                  MaterialPageRoute(builder: (context) => SignupPage()),
                );
              },
              child: Text('Sign up'),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildWelcomePage(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('Main page'),
      ),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text(
              'Hello, $username!',
              style: TextStyle(fontSize: 24),
            ),
            ElevatedButton(
              onPressed: _logout,
              child: Text('Log out'),
            ),
          ],
        ),
      ),
    );
  }
}
